package be.asmolabs.palettier.ui.picker

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import be.asmolabs.palettier.domain.color.Rgb
import be.asmolabs.palettier.domain.color.neutralise
import be.asmolabs.palettier.domain.image.ImagePalette
import be.asmolabs.palettier.domain.mix.ColorMixService
import be.asmolabs.palettier.domain.plan.SampledPlanService
import be.asmolabs.palettier.domain.plan.asProject
import be.asmolabs.palettier.domain.plan.uniqueName
import be.asmolabs.palettier.domain.port.ProjectRepository
import be.asmolabs.palettier.domain.image.PixelMap
import be.asmolabs.palettier.domain.image.sampleAround
import be.asmolabs.palettier.domain.paint.findClosest
import be.asmolabs.palettier.domain.port.PaintCatalogRepository
import be.asmolabs.palettier.domain.port.PaletteRepository
import be.asmolabs.palettier.image.ImageDecoder
import be.asmolabs.palettier.ui.SampledColour
import be.asmolabs.palettier.ui.candidates
import be.asmolabs.palettier.ui.scopesOf
import kotlinx.coroutines.Job
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
    private val sampled: SampledColour,
    private val mixer: ColorMixService,
    private val projects: ProjectRepository,
    private val sampledPlans: SampledPlanService,
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

    /** La recherche de melange : elle coute, donc une nouvelle demande annule l'ancienne. */
    private var search: Job? = null

    init {
        // Les palettes sont lues une fois : ce sont des portees de recherche, pas une
        // donnee que l'ecran affiche et qui devrait se rafraichir toute seule.
        viewModelScope.launch {
            val known = palettes.all()
            _state.value = _state.value.copy(
                scopes = scopesOf(known),
                palettes = known,
                projectPaletteId = known.firstOrNull()?.id,
            )
        }
    }

    fun onIntent(intent: PickerIntent) {
        when (intent) {
            is PickerIntent.Load -> viewModelScope.launch { load(intent.bytes) }

            // Les propositions sont videes avant de recalculer : les garder afficherait,
            // le temps du calcul, des tubes choisis pour une autre teinte. Bref, mais faux.
            is PickerIntent.Select -> viewModelScope.launch {
                _state.value = _state.value.copy(selectedHex = intent.hex, closest = emptyList())
                forgetMixes()
                recompute()
            }

            is PickerIntent.UseAsGrey -> viewModelScope.launch {
                _state.value = _state.value.copy(greyPointHex = intent.hex, closest = emptyList())
                forgetMixes()
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

            PickerIntent.FindMixes -> findMixes()

            // Changer le nombre de tubes repose la question : on relance, mais seulement
            // si une reponse avait deja ete demandee.
            is PickerIntent.SetMaxPaints -> {
                _state.value = _state.value.copy(maxPaints = intent.count.coerceIn(1, 4))
                if (_state.value.suggestions.isNotEmpty() || _state.value.searching) findMixes()
            }


            // --- Les groupes de releves -----------------------------------

            is PickerIntent.CreateGroup -> {
                val name = intent.name.trim()
                _state.value = when {
                    name.isEmpty() ->
                        _state.value.copy(groupMessage = "Donnez un nom au groupe.")
                    _state.value.groups.any { it.name.equals(name, ignoreCase = true) } ->
                        _state.value.copy(groupMessage = "Un groupe porte deja ce nom.")
                    else -> _state.value.copy(
                        groups = _state.value.groups + SampleGroup(name),
                        selectedGroup = name,
                        groupMessage = "Relevez une teinte sur la photo, puis ajoutez-la.",
                    )
                }
            }

            is PickerIntent.SelectGroup ->
                _state.value = _state.value.copy(selectedGroup = intent.name, groupMessage = null)

            is PickerIntent.DeleteGroup -> {
                val left = _state.value.groups.filterNot { it.name == intent.name }
                _state.value = _state.value.copy(
                    groups = left,
                    selectedGroup = _state.value.selectedGroup?.takeIf { it != intent.name }
                        ?: left.firstOrNull()?.name,
                    groupMessage = null,
                )
            }

            PickerIntent.AddSampleToGroup -> {
                val current = _state.value
                val colour = current.workingHex?.let { runCatching { Rgb.ofHex(it) }.getOrNull() }
                val group = current.group
                _state.value = when {
                    group == null -> current.copy(groupMessage = "Choisissez d'abord un groupe.")
                    colour == null -> current.copy(groupMessage = "Relevez d'abord une teinte.")
                    else -> {
                        val grown = group.copy(samples = group.samples + colour)
                        current.copy(
                            groups = current.groups.map { if (it.name == group.name) grown else it },
                            groupMessage = "${grown.samples.size} point" +
                                (if (grown.samples.size > 1) "s releves." else " releve."),
                        )
                    }
                }
            }

            is PickerIntent.RemoveSample -> {
                _state.value = _state.value.copy(
                    groups = _state.value.groups.map { group ->
                        if (group.name != intent.group) {
                            group
                        } else {
                            // Un seul exemplaire retire : deux points peuvent tomber sur la
                            // meme teinte sans que l'un annule l'autre.
                            val index = group.samples.indexOfFirst { it.toHex() == intent.hex }
                            if (index < 0) group
                            else group.copy(samples = group.samples.filterIndexed { i, _ -> i != index })
                        }
                    },
                    groupMessage = null,
                )
            }

            is PickerIntent.ChooseProjectPalette ->
                _state.value = _state.value.copy(projectPaletteId = intent.id)

            is PickerIntent.CreateProject -> viewModelScope.launch { createProject(intent.name) }

            is PickerIntent.ChooseScope -> viewModelScope.launch {
                _state.value = _state.value.copy(scope = intent.scope, closest = emptyList())
                forgetMixes()
                recompute()
            }
        }
    }

    private suspend fun load(bytes: ByteArray) {
        // Ce qui appartient a la photo est remis a zero ; ce qui appartient a la session
        // reste. Les groupes en particulier : on releve volontiers la peau sur une
        // reference et la cape sur une autre, et les perdre en ouvrant la seconde
        // ruinerait le travail de la premiere.
        _state.value = _state.value.copy(
            loading = true,
            error = null,
            image = null,
            dominant = emptyList(),
            selectedHex = null,
            pickedAt = null,
            pickingGrey = false,
            greyPointHex = null,
            correctedHex = null,
            closest = emptyList(),
            suggestions = emptyList(),
            searching = false,
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
            _state.value = _state.value.copy(
                loading = false,
                image = null,
                dominant = emptyList(),
                error = e.message ?: "Image illisible",
            )
        }
    }

    /**
     * Releve la teinte sous le clic, et l'attribue selon ce que le peintre a arme.
     *
     * <p>Un clic hors de l'image ne fait rien plutot que de rendre du noir : l'ecran peut
     * afficher une photo dans une zone plus large qu'elle.</p>
     */
    private suspend fun pick(x: Int, y: Int) {
        val sampled = pixels?.sampleAround(x, y, _state.value.sampleRadius) ?: return
        val hex = sampled.toHex()

        // L'oubli d'abord, l'etat ensuite : lire l'etat avant de l'oublier reintroduirait
        // les melanges par la copie qui suit. C'est un essai qui l'a dit.
        forgetMixes()
        val current = _state.value
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

        // La pipette mesure, les autres ecrans s'en servent. Elle publie donc la teinte
        // de travail -- corrigee si elle l'a ete, puisque c'est celle qui vaut.
        sampled.set(working)

        _state.value = current.copy(
            correctedHex = grey?.let { working.toHex() },
            emptyScope = candidates.isEmpty(),
            closest = findClosest(working, candidates, CLOSEST),
        )
    }

    /**
     * Cherche le melange qui donne la teinte retenue.
     *
     * <p>Sur demande, et non a chaque clic : l'exploration des dosages realisables coute
     * plusieurs centaines de millisecondes sur six cent quatre-vingts tubes, et le peintre
     * clique souvent plusieurs fois avant de retenir un point.</p>
     */
    private fun findMixes() {
        val current = _state.value
        val target = current.workingHex?.let { runCatching { Rgb.ofHex(it) }.getOrNull() } ?: return

        search?.cancel()
        _state.value = current.copy(suggestions = emptyList(), searching = true)
        search = viewModelScope.launch {
            val candidates = _state.value.scope.candidates(catalog, palettes)
            val found = mixer.suggestMixes(
                target,
                candidates,
                maxResults = MIXES,
                maxPaints = _state.value.maxPaints,
            )
            _state.value = _state.value.copy(suggestions = found, searching = false)
        }
    }

    /**
     * Oublie les melanges proposes : ils repondaient a une autre question.
     *
     * <p>Les garder le temps d'un nouveau releve afficherait une recette pour la teinte
     * precedente. Bref, et faux.</p>
     */
    private fun forgetMixes() {
        search?.cancel()
        _state.value = _state.value.copy(suggestions = emptyList(), searching = false)
    }

/**
     * Etablit le plan a partir des groupes, et l'enregistre comme projet.
     *
     * <p>Un nom deja pris est numerote plutot que refuse : deux bustes du meme regiment
     * s'appellent volontiers pareil, et perdre le plan qu'on vient d'etablir pour si peu
     * serait cruel.</p>
     */
    private suspend fun createProject(wanted: String) {
        val current = _state.value
        val definition = current.groups
            .filter { it.samples.isNotEmpty() }
            .map { SampledPlanService.ColourGroup(it.name, samples = it.samples) }

        if (definition.isEmpty()) {
            _state.value = current.copy(groupMessage = "Aucun point releve : ajoutez une teinte a un groupe.")
            return
        }

        _state.value = current.copy(savingProject = true, groupMessage = null)
        try {
            val palette = current.palettes.firstOrNull { it.id == current.projectPaletteId }
            val plan = sampledPlans.plan(wanted.trim(), palette, definition, current.maxPaints)
            val name = uniqueName(wanted, projects.all().map { it.name })
            val saved = projects.save(plan.asProject(name, palette))

            _state.value = _state.value.copy(
                savingProject = false,
                groupMessage = "Projet \"${saved.name}\" cree a partir de ${plan.zones.size} " +
                    (if (plan.zones.size > 1) "groupes." else "groupe.") +
                    " Retrouvez-le dans Projets.",
            )
        } catch (e: Exception) {
            _state.value = _state.value.copy(
                savingProject = false,
                groupMessage = e.message ?: "Le projet n'a pas pu etre enregistre.",
            )
        }
    }

    companion object {
        /** Teintes relevees sur la photo : assez pour couvrir une piece, pas pour noyer. */
        private const val COLOURS = 8

        /** Tubes proposes : au-dela, on ne compare plus, on parcourt. */
        private const val CLOSEST = 6

        /** Rayon maximal, comme du temps de JavaFX : au-dela on ne releve plus, on moyenne. */
        const val MAX_RADIUS = 30

        /** Melanges proposes : au-dela, on ne choisit plus, on compare des variantes. */
        private const val MIXES = 5
    }
}
