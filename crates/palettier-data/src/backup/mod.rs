//! Portable ZIP backup archive model, reader/writer, importer, and exporter.

mod archive;
mod exporter;
mod importer;
mod model;

use chrono::{DateTime, Utc};

pub use archive::{BackupArchive, BackupWriter, ZipBackupArchive, ZipBackupWriter};
pub use exporter::{BackupExporter, ExportReport};
pub use importer::{BackupImporter, ImportReport};
pub use model::{
    BackupDocument, BackupLayer, BackupPaint, BackupPaintRef, BackupPalette, BackupPhoto,
    BackupProject, BackupRecipe, BackupStep, BackupZone,
};

/// Studio UI preference keys safely included in backup archives (never AI secrets/keys).
pub(crate) const STUDIO_SETTING_KEYS: [&str; 4] =
    ["ui.locale", "ui.theme", "ui.paint_system", "ui.workshop"];

pub(crate) fn parse_iso_instant(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

pub(crate) fn format_iso_instant(dt: DateTime<Utc>) -> String {
    dt.to_rfc3339_opts(chrono::SecondsFormat::AutoSi, true)
}
