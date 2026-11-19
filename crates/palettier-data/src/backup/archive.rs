use super::exporter::BackupExporter;
use chrono::Utc;
use indexmap::IndexMap;
use palettier_domain::error::{DomainError, DomainResult};
use palettier_domain::i18n::Locale;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use zip::ZipArchive;
use zip::write::SimpleFileOptions;

/// Maximum accepted size for `palettier.json` (32 MB) to guard against Zip Bombs.
const MAX_JSON_BYTES: u64 = 32 * 1024 * 1024;
/// Maximum accepted size per image in an archive (50 MB).
const MAX_ENTRY_BYTES: u64 = 50 * 1024 * 1024;
/// Maximum cumulative decompressed bytes read from a single archive (256 MB).
const MAX_TOTAL_ARCHIVE_BYTES: u64 = 256 * 1024 * 1024;

pub trait BackupArchive {
    fn document(&mut self) -> DomainResult<Vec<u8>>;
    fn file(&mut self, path: &str) -> DomainResult<Option<Vec<u8>>>;
}

pub trait BackupWriter {
    fn write_document(&mut self, json: String);
    fn write_file(&mut self, path: String, bytes: Vec<u8>);
    fn finish(self) -> DomainResult<()>;
}

pub struct ZipBackupArchive {
    zip: ZipArchive<File>,
    total_read_bytes: u64,
    locale: Locale,
}

impl ZipBackupArchive {
    pub const DOCUMENT: &'static str = "palettier.json";

    pub fn open(path: impl AsRef<Path>) -> DomainResult<Self> {
        Self::open_in(path, Locale::Fr)
    }

    pub fn open_in(path: impl AsRef<Path>, locale: Locale) -> DomainResult<Self> {
        let file = File::open(path.as_ref()).map_err(|e| {
            let msg = match locale {
                Locale::Fr => format!("Impossible d'ouvrir l'archive {:?}: {e}", path.as_ref()),
                Locale::En => format!("Unable to open archive {:?}: {e}", path.as_ref()),
            };
            DomainError::Persistence(msg)
        })?;
        let zip = ZipArchive::new(file).map_err(|e| {
            let msg = match locale {
                Locale::Fr => format!("Archive ZIP invalide {:?}: {e}", path.as_ref()),
                Locale::En => format!("Invalid ZIP archive {:?}: {e}", path.as_ref()),
            };
            DomainError::Persistence(msg)
        })?;
        Ok(Self {
            zip,
            total_read_bytes: 0,
            locale,
        })
    }

    fn read_bounded(
        mut entry: zip::read::ZipFile<'_, File>,
        max_bytes: u64,
        total_read_bytes: &mut u64,
        locale: Locale,
    ) -> DomainResult<Vec<u8>> {
        // Zip-Slip protection: reject paths containing `..` or absolute paths.
        if entry.enclosed_name().is_none() {
            let msg = match locale {
                Locale::Fr => format!("Chemin d'entrée ZIP dangereux rejeté : {:?}", entry.name()),
                Locale::En => format!("Unsafe ZIP entry path rejected: {:?}", entry.name()),
            };
            return Err(DomainError::InvalidArgument(msg));
        }
        let remaining_budget = MAX_TOTAL_ARCHIVE_BYTES.saturating_sub(*total_read_bytes);
        let effective_limit = max_bytes.min(remaining_budget);
        if entry.size() > effective_limit {
            let msg = match locale {
                Locale::Fr => format!(
                    "Entrée ZIP trop volumineuse ou budget d'archive dépassé ({} octets > limite {})",
                    entry.size(),
                    effective_limit
                ),
                Locale::En => format!(
                    "ZIP entry too large or archive budget exceeded ({} bytes > limit {})",
                    entry.size(),
                    effective_limit
                ),
            };
            return Err(DomainError::InvalidArgument(msg));
        }
        let mut buf = Vec::with_capacity(entry.size().min(1024 * 1024) as usize);
        let mut limited = (&mut entry).take(effective_limit + 1);
        limited.read_to_end(&mut buf).map_err(|e| {
            let msg = match locale {
                Locale::Fr => format!("Erreur de lecture ZIP: {e}"),
                Locale::En => format!("ZIP read error: {e}"),
            };
            DomainError::Persistence(msg)
        })?;
        if (buf.len() as u64) > effective_limit {
            let msg = match locale {
                Locale::Fr => format!(
                    "Entrée ZIP dépassant la taille maximale autorisée ({effective_limit} octets)"
                ),
                Locale::En => format!(
                    "ZIP entry exceeds maximum allowed size ({effective_limit} bytes)"
                ),
            };
            return Err(DomainError::InvalidArgument(msg));
        }
        *total_read_bytes = total_read_bytes.saturating_add(buf.len() as u64);
        Ok(buf)
    }
}

impl BackupArchive for ZipBackupArchive {
    fn document(&mut self) -> DomainResult<Vec<u8>> {
        let mut total = self.total_read_bytes;
        let locale = self.locale;
        let res = {
            let entry = self.zip.by_name(Self::DOCUMENT).map_err(|_| {
                let msg = match locale {
                    Locale::Fr => format!(
                        "Cette archive ne contient pas de {} : ce n'est pas une sauvegarde Palettier.",
                        Self::DOCUMENT
                    ),
                    Locale::En => format!(
                        "This archive does not contain {}: it is not a Palettier backup.",
                        Self::DOCUMENT
                    ),
                };
                DomainError::InvalidArgument(msg)
            })?;
            Self::read_bounded(entry, MAX_JSON_BYTES, &mut total, locale)
        };
        self.total_read_bytes = total;
        res
    }

    fn file(&mut self, path: &str) -> DomainResult<Option<Vec<u8>>> {
        let locale = self.locale;
        if Path::new(path).components().any(|c| {
            matches!(
                c,
                std::path::Component::ParentDir | std::path::Component::RootDir
            )
        }) {
            let msg = match locale {
                Locale::Fr => format!("Chemin d'archive non sécurisé : {path}"),
                Locale::En => format!("Unsafe archive path: {path}"),
            };
            return Err(DomainError::InvalidArgument(msg));
        }
        let mut total = self.total_read_bytes;
        let res = match self.zip.by_name(path) {
            Ok(entry) => Ok(Some(Self::read_bounded(
                entry,
                MAX_ENTRY_BYTES,
                &mut total,
                locale,
            )?)),
            Err(zip::result::ZipError::FileNotFound) => Ok(None),
            Err(e) => {
                let msg = match locale {
                    Locale::Fr => format!("Erreur d'accès à l'entrée ZIP {path}: {e}"),
                    Locale::En => format!("Error reading ZIP entry {path}: {e}"),
                };
                Err(DomainError::Persistence(msg))
            }
        };
        self.total_read_bytes = total;
        res
    }
}

pub struct ZipBackupWriter {
    archive_path: PathBuf,
    document: Option<String>,
    files: IndexMap<String, Vec<u8>>,
}

impl ZipBackupWriter {
    pub fn new(archive_path: impl AsRef<Path>) -> Self {
        Self {
            archive_path: archive_path.as_ref().to_path_buf(),
            document: None,
            files: IndexMap::new(),
        }
    }
}

impl BackupWriter for ZipBackupWriter {
    fn write_document(&mut self, json: String) {
        self.document = Some(json);
    }

    fn write_file(&mut self, path: String, bytes: Vec<u8>) {
        self.files.insert(path, bytes);
    }

    fn finish(self) -> DomainResult<()> {
        let Some(json) = self.document else {
            return Ok(());
        };
        let parent = self
            .archive_path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        std::fs::create_dir_all(parent).map_err(|e| {
            DomainError::Persistence(format!("Impossible de créer le dossier {:?}: {e}", parent))
        })?;

        let file_name = self
            .archive_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("backup.zip");
        let temp_path = parent.join(format!(
            ".{file_name}.tmp-{}-{}",
            std::process::id(),
            Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));

        let write_result = (|| -> DomainResult<()> {
            let file = File::create(&temp_path).map_err(|e| {
                DomainError::Persistence(format!(
                    "Impossible de créer l'archive temporaire {:?}: {e}",
                    temp_path
                ))
            })?;
            let mut zip = zip::ZipWriter::new(file);
            let options =
                SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

            zip.start_file(BackupExporter::DOCUMENT, options)
                .map_err(|e| DomainError::Persistence(format!("Erreur ZIP: {e}")))?;
            zip.write_all(json.as_bytes())
                .map_err(|e| DomainError::Persistence(format!("Erreur d'écriture ZIP: {e}")))?;

            for (path, bytes) in self.files {
                zip.start_file(path, options)
                    .map_err(|e| DomainError::Persistence(format!("Erreur ZIP: {e}")))?;
                zip.write_all(&bytes)
                    .map_err(|e| DomainError::Persistence(format!("Erreur d'écriture ZIP: {e}")))?;
            }

            let finished_file = zip
                .finish()
                .map_err(|e| DomainError::Persistence(format!("Erreur de clôture ZIP: {e}")))?;
            finished_file
                .sync_all()
                .map_err(|e| DomainError::Persistence(format!("Erreur de synchronisation ZIP: {e}")))?;
            Ok(())
        })();

        if let Err(err) = write_result {
            let _ = std::fs::remove_file(&temp_path);
            return Err(err);
        }

        if let Err(err) = std::fs::rename(&temp_path, &self.archive_path) {
            let _ = std::fs::remove_file(&temp_path);
            return Err(DomainError::Persistence(format!(
                "Impossible de finaliser l'archive {:?}: {err}",
                self.archive_path
            )));
        }

        Ok(())
    }
}
