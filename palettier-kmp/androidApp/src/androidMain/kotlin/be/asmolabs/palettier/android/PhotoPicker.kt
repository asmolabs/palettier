package be.asmolabs.palettier.android

import android.content.Context
import android.net.Uri
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.ActivityResultLauncher
import androidx.activity.result.PickVisualMediaRequest
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.platform.LocalContext
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

/**
 * Le choix d'une photo, ramene a des octets.
 *
 * <p>Le selecteur du systeme est employe plutot qu'une permission de lecture : il rend
 * l'image choisie sans donner acces au reste de la galerie, ce qui correspond a ce que
 * l'application promet -- elle lit ce qu'on lui montre, rien d'autre.</p>
 *
 * <p>Le fichier est lu tel quel, non reduit. La mesure de la pipette porte sur les pixels
 * d'origine : reduire reencode, et reencoder deplace les couleurs.</p>
 */
class PhotoPicker(private val context: Context) {

    /**
     * Repose a chaque composition : le lanceur est lie au cycle de vie de l'Activity, et
     * en garder un d'avant une rotation d'ecran le rendrait muet.
     */
    internal var launcher: ActivityResultLauncher<PickVisualMediaRequest>? = null

    private var pending: CompletableDeferred<Uri?>? = null

    /** @return les octets du fichier choisi, ou null si le peintre a renonce */
    suspend fun pick(): ByteArray? {
        val launcher = launcher ?: return null

        // Une demande a la fois : deux selecteurs ouverts ensemble n'auraient aucun sens,
        // et la precedente ne recevrait de toute facon jamais de reponse.
        pending?.complete(null)
        val awaited = CompletableDeferred<Uri?>()
        pending = awaited

        launcher.launch(PickVisualMediaRequest(ActivityResultContracts.PickVisualMedia.ImageOnly))
        val uri = awaited.await() ?: return null

        return withContext(Dispatchers.IO) {
            runCatching { context.contentResolver.openInputStream(uri)?.readBytes() }.getOrNull()
        }
    }

    internal fun resolve(uri: Uri?) {
        pending?.complete(uri)
        pending = null
    }
}

@Composable
fun rememberPhotoPicker(): PhotoPicker {
    val context = LocalContext.current
    val picker = remember(context) { PhotoPicker(context) }

    picker.launcher = rememberLauncherForActivityResult(
        ActivityResultContracts.PickVisualMedia(),
    ) { uri -> picker.resolve(uri) }

    return picker
}
