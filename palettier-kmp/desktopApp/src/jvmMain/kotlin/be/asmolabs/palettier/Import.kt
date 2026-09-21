package be.asmolabs.palettier

import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import be.asmolabs.palettier.data.ZipBackupArchive
import be.asmolabs.palettier.data.ZipBackupWriter
import be.asmolabs.palettier.data.backup.BackupExporter
import be.asmolabs.palettier.data.backup.BackupImporter
import be.asmolabs.palettier.data.backup.ExportReport
import be.asmolabs.palettier.data.backup.ImportReport
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.awt.FileDialog
import java.awt.Frame
import java.nio.file.Path

/**
 * L'import d'une sauvegarde, depuis le bureau.
 *
 * <p>Le choix du fichier appartient a la plateforme, d'ou sa place ici plutot que dans
 * feature-ui. C'est aussi le seul chemin par lequel les donnees de l'application Java
 * entrent : tant qu'on ne l'a pas emprunte, la base est vide et l'ecran le dit.</p>
 */
class ImportState(
    private val importer: BackupImporter,
    private val exporter: BackupExporter,
    private val scope: CoroutineScope,
) {

    var busy by mutableStateOf(false)
        private set

    /** Ce qu'on affiche apres coup : le rapport, ou l'echec. */
    var message by mutableStateOf<String?>(null)
        private set

    fun choose() {
        if (busy) return
        val file = pickArchive() ?: return

        busy = true
        message = null
        scope.launch {
            message = try {
                val report = withContext(Dispatchers.IO) {
                    ZipBackupArchive(file).use { importer.import(it) }
                }
                describe(report)
            } catch (e: Exception) {
                "Import impossible : ${e.message}"
            }
            busy = false
        }
    }

    /** Ecrit une archive la ou le peintre la range. */
    fun exportTo() {
        if (busy) return
        val file = chooseDestination() ?: return

        busy = true
        message = null
        scope.launch {
            message = try {
                val report = withContext(Dispatchers.IO) {
                    ZipBackupWriter(file).use { exporter.export(it) }
                }
                describe(report, file)
            } catch (e: Exception) {
                "Export impossible : ${e.message}"
            }
            busy = false
        }
    }

    private fun describe(report: ExportReport, file: Path): String =
        "${report.paints} tubes, ${report.palettes} palettes, ${report.projects} projets, " +
            "${report.recipes} recettes, ${report.photos} photos ecrits dans ${file.fileName}"

    private fun chooseDestination(): Path? {
        val dialog = FileDialog(null as Frame?, "Enregistrer la sauvegarde", FileDialog.SAVE)
        dialog.file = "palettier-sauvegarde.zip"
        dialog.isVisible = true
        val directory = dialog.directory ?: return null
        val name = dialog.file ?: return null
        return Path.of(directory, if (name.endsWith(".zip")) name else "$name.zip")
    }

    private fun pickArchive(): Path? {
        val dialog = FileDialog(null as Frame?, "Sauvegarde Palettier a importer", FileDialog.LOAD)
        dialog.setFilenameFilter { _, name -> name.endsWith(".zip") }
        dialog.isVisible = true
        val directory = dialog.directory ?: return null
        val name = dialog.file ?: return null
        return Path.of(directory, name)
    }

    /**
     * Le rapport en clair. Ce qui a ete laisse de cote compte autant que ce qui est
     * arrive : une restauration qui ne dit pas ce qu'elle a ignore laisse un doute.
     */
    private fun describe(report: ImportReport): String = buildString {
        append("${report.paintsCreated} tubes ajoutes, ${report.paintsUpdated} mis a jour")
        append(" - ${report.palettesAdded} palettes, ${report.projectsAdded} projets, ")
        append("${report.recipesAdded} recettes, ${report.photos} photos")
        if (report.skipped > 0) {
            append(" - ${report.skipped} laisses de cote, un travail du meme nom existait deja")
        }
        report.warnings.take(2).forEach { append("\n$it") }
    }
}
