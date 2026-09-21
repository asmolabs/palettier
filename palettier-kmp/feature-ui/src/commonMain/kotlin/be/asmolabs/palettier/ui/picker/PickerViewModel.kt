package be.asmolabs.palettier.ui.picker

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import be.asmolabs.palettier.domain.color.Rgb
import be.asmolabs.palettier.domain.color.neutralise
import be.asmolabs.palettier.domain.image.ImagePalette
import be.asmolabs.palettier.domain.image.PixelMap
import be.asmolabs.palettier.domain.image.sampleAround
import be.asmolabs.palettier.domain.paint.findClosest
import be.asmolabs.palettier.domain.port.PaintCatalogRepository
import be.asmolabs.palettier.domain.port.PaletteRepository
import be.asmolabs.palettier.image.ImageDecoder
import be.asmolabs.palettier.ui.candidates
import be.asmolabs.palettier.ui.scopesOf
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

/**
 * La pipette : relever une teinte sur une photo, et voir avec quoi la faire.
 *
 * <p>Deux precautions y sont incorporees, parce que sans elles le releve ment.</p>
 *
 * <p>La mesure se fait sur le fichier d'origine, jamais sur une image reduite : reduire
 * reencode, et reencoder deplace les couleurs.</p>
 *
 * <p>Et une photo prise sous lampe de bureau tire au jaune, sous LED froide au bleu.
 * Designer un point cense etre gris donne le gain a appliquer a chaque canal pour annuler
 * cette dominante -- la balance des blancs du photographe, reduite a son necessaire.</p>
 */
class PickerViewModel(
    private val decoder: ImageDecoder,
    private val catalog: PaintCatalogRepository,
    private val palettes: PaletteRepository,
) : ViewModel() {

    private val _state = MutableStateFlow(PickerUiState())
    val state: StateFlow<PickerUiState> = _state.asStateFlow()

    /**
     * Les pixels de la photo ouverte, gardes hors de l'etat.
     *
     * <p>Ils ne servent qu'ici, et les mettre dans l'etat ferait comparer un tableau de
     * plusieurs millions d'entiers a chaque emission du StateFlow. Ce sont ceux du
     * fichier d'origine, jamais d'une image reduite : reduire reencode, et reencoder
     * deplace les couleurs.</p>
     */
    private var pixels: PixelMap? = null

    init {
        // Les palettes sont lues une fois : ce sont des portees de recherche, pas une
        // donnee que l'ecran affiche et qui devrait se rafraichir toute seule.
        viewModelScope.launch {
            _state.value = _state.value.copy(scopes = scopesOf(palettes.all()))
        }
    }

    fun onIntent(intent: PickerIntent) {
        when (intent) {
            is PickerIntent.Load -> viewModelScope.launch { load(intent.bytes) }

            // Les propositions sont videes avant de recalculer : les garder afficherait,
            // le temps du calcul, des tubes choisis pour une autre teinte. Bref, mais faux.
            is PickerIntent.Select -> viewModelScope.launch {
                _state.value = _state.value.copy(selectedHex = intent.hex, closest = emptyList())
                recompute()
            }

            is PickerIntent.UseAsGrey -> viewModelScope.launch {
                _state.value = _state.value.copy(greyPointHex = intent.hex, closest = emptyList())
                recompute()
            }

            is PickerIntent.PickAt -> viewModelScope.launch { pick(intent.x, intent.y) }

            is PickerIntent.PickGrey -> {
                _state.value = _state.value.copy(pickingGrey = intent.arming)
            }

            // Le rayon ne redeclenche rien : il vaut pour le prochain releve. Recalculer
            // celui deja pris demanderait de se souvenir d'ou, pour un resultat que le
            // peintre n'a pas demande.
            is PickerIntent.SetRadius -> {
                _state.value = _state.value.copy(sampleRadius = intent.radius.coerceIn(0, MAX_RADIUS))
            }

            is PickerIntent.ChooseScope -> viewModelScope.launch {
                _state.value = _state.value.copy(scope = intent.scope, closest = emptyList())
                recompute()
            }
        }
    }

    private suspend fun load(bytes: ByteArray) {
        val kept = _state.value
        _state.value = PickerUiState(
            loading = true,
            scope = kept.scope,
            scopes = kept.scopes,
            sampleRadius = kept.sampleRadius,
        )
        try {
            val decoded = decoder.decode(bytes)
            pixels = decoded
            val dominant = ImagePalette.dominant(decoded, COLOURS)
            _state.value = _state.value.copy(
                loading = false,
                image = LoadedImage(bytes, decoded.width, decoded.height),
                dominant = dominant,
                selectedHex = dominant.firstOrNull()?.color?.toHex(),
            )
            recompute()
        } catch (e: Exception) {
            // Large a dessein : un fichier douteux ne doit pas faire tomber l'ecran, et
            // les facons d'echouer a lire une image ne se laissent pas enumerer.
            pixels = null
            _state.value = PickerUiState(error = e.message ?: "Image illisible")
        }
    }

    /**
     * Releve la teinte sous le clic, et l'attribue selon ce que le peintre a arme.
     *
     * <p>Un clic hors de l'image ne fait rien plutot que de rendre du noir : l'ecran peut
     * afficher une photo dans une zone plus large qu'elle.</p>
     */
    private suspend fun pick(x: Int, y: Int) {
        val current = _state.value
        val sampled = pixels?.sampleAround(x, y, current.sampleRadius) ?: return
        val hex = sampled.toHex()

        _state.value = if (current.pickingGrey) {
            // Le point neutre se desarme apres usage : on en designe un, pas une suite.
            current.copy(greyPointHex = hex, pickingGrey = false, closest = emptyList())
        } else {
            current.copy(selectedHex = hex, pickedAt = PickPoint(x, y), closest = emptyList())
        }
        recompute()
    }

    /**
     * Applique la correction de dominante s'il y en a une, puis cherche les tubes les
     * plus proches.
     */
    private suspend fun recompute() {
        val current = _state.value
        val selected = current.selectedHex?.let { runCatching { Rgb.ofHex(it) }.getOrNull() }
            ?: return

        val grey = current.greyPointHex?.let { runCatching { Rgb.ofHex(it) }.getOrNull() }
        val working = grey?.let { neutralise(selected, it) } ?: selected

        val candidates = current.scope.candidates(catalog, palettes)

        _state.value = current.copy(
            correctedHex = grey?.let { working.toHex() },
            emptyScope = candidates.isEmpty(),
            closest = findClosest(working, candidates, CLOSEST),
        )
    }

    companion object {
        /** Teintes relevees sur la photo : assez pour couvrir une piece, pas pour noyer. */
        private const val COLOURS = 8

        /** Tubes proposes : au-dela, on ne compare plus, on parcourt. */
        private const val CLOSEST = 6

        /** Rayon maximal, comme du temps de JavaFX : au-dela on ne releve plus, on moyenne. */
        const val MAX_RADIUS = 30
    }
}
