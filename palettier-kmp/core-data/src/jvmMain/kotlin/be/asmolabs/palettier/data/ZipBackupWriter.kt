package be.asmolabs.palettier.data

import be.asmolabs.palettier.data.backup.BackupExporter
import be.asmolabs.palettier.data.backup.BackupWriter
import java.nio.file.Files
import java.nio.file.Path
import java.util.zip.ZipEntry
import java.util.zip.ZipOutputStream

/**
 * Ecrit l'archive en ZIP, comme l'application Java.
 *
 * <p>Le document est ecrit en dernier, quelle que soit l'ordre des appels : il annonce
 * les images, et une archive ou il precederait des fichiers manquants serait illisible a
 * moitie plutot que franchement invalide.</p>
 */
class ZipBackupWriter(private val archive: Path) : BackupWriter, AutoCloseable {

    private var document: String? = null
    private val files = LinkedHashMap<String, ByteArray>()

    override suspend fun writeDocument(json: String) {
        document = json
    }

    override suspend fun writeFile(path: String, bytes: ByteArray) {
        files[path] = bytes
    }

    /** Ecrit reellement l'archive. Rien n'est touche sur le disque avant cet appel. */
    override fun close() {
        val json = document ?: return
        Files.createDirectories(archive.toAbsolutePath().parent)

        ZipOutputStream(Files.newOutputStream(archive)).use { zip ->
            zip.putNextEntry(ZipEntry(BackupExporter.DOCUMENT))
            zip.write(json.toByteArray())
            zip.closeEntry()

            files.forEach { (path, bytes) ->
                zip.putNextEntry(ZipEntry(path))
                zip.write(bytes)
                zip.closeEntry()
            }
        }
    }
}
