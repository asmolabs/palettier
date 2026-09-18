package be.asmolabs.palettier.data

import be.asmolabs.palettier.data.backup.BackupArchive
import java.nio.file.Path
import java.util.zip.ZipFile

/**
 * Une sauvegarde Palettier, telle que l'application Java la produit : un ZIP contenant un
 * JSON et les images rangees a cote.
 *
 * <p>Implementation JVM seulement, et c'est suffisant : la sauvegarde est produite sur le
 * poste de bureau, c'est donc la qu'elle doit se relire. Les cibles mobiles n'en auront
 * besoin que le jour ou l'on voudra y transferer des donnees, ce qui est une autre
 * question -- Palettier ne synchronise rien.</p>
 */
class ZipBackupArchive(private val archive: Path) : BackupArchive, AutoCloseable {

    private val zip = ZipFile(archive.toFile())

    override fun document(): ByteArray {
        val entry = zip.getEntry(DOCUMENT)
            ?: throw IllegalArgumentException(
                "Cette archive ne contient pas de $DOCUMENT : ce n'est pas une sauvegarde Palettier."
            )
        return zip.getInputStream(entry).readAllBytes()
    }

    override fun file(path: String): ByteArray? =
        zip.getEntry(path)?.let { zip.getInputStream(it).readAllBytes() }

    override fun close() = zip.close()

    companion object {
        const val DOCUMENT = "palettier.json"
    }
}
