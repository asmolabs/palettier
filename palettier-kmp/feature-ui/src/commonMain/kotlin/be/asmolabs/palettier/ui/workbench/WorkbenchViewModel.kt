package be.asmolabs.palettier.ui.workbench

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import be.asmolabs.palettier.domain.color.Rgb
import be.asmolabs.palettier.domain.drying.Workshop
import be.asmolabs.palettier.domain.palette.PaletteMix
import be.asmolabs.palettier.domain.palette.PaletteMixService
import be.asmolabs.palettier.domain.port.PaletteMixRepository
import be.asmolabs.palettier.domain.port.ProjectRepository
import be.asmolabs.palettier.domain.workbench.WorkbenchService
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import kotlin.time.Clock
import kotlin.time.Duration
import kotlin.time.Duration.Companion.minutes

/**
 * L'etat de l'etabli, tenu a jour tout seul.
 *
 * <p>L'ecran change pour deux raisons independantes : on coche une couche, ou le temps
 * passe. La premiere vient du flux de la base, la seconde d'un battement. Les combiner
 * supprime tout le rafraichissement manuel -- la minuterie, le bouton, l'ecouteur sur la
 * scene et les appels apres chaque ecriture que l'application JavaFX devait cabler un par
 * un. Une couche cochee dans l'ecran Projets rafraichit celui-ci sans que personne ne
 * relie les deux.</p>
 *
 * <p>Le partage s'arrete cinq secondes apres le dernier abonne. La regle "ne pas relire la
 * base pour un ecran que personne ne regarde" devient une propriete du cadre, au lieu d'un
 * else a ne pas oublier.</p>
 */
@OptIn(ExperimentalCoroutinesApi::class)
class WorkbenchViewModel(
    projects: ProjectRepository,
    private val mixesRepository: PaletteMixRepository,
    private val workbench: WorkbenchService,
    private val mixes: PaletteMixService,
    private val clock: Clock = Clock.System,
    private val tick: Duration = 1.minutes,
    /** Conditions de l'atelier employees pour un melange qu'on pose. */
    private val workshop: () -> Workshop = { Workshop.standard() },
) : ViewModel() {

    private val sampled = MutableStateFlow<Rgb?>(null)

    val state: StateFlow<WorkbenchUiState> =
        combine(
            projects.observeAll(),
            mixesRepository.observeAll(),
            sampled,
            ticker(tick),
        ) { pieces, palette, colour, _ ->
            val now = clock.now()
            WorkbenchUiState(
                loading = false,
                bench = workbench.bench(pieces, now),
                mixes = mixes.states(palette, now),
                sampledHex = colour?.toHex(),
            )
        }
            .catch { emit(WorkbenchUiState(loading = false, error = it.message ?: "Lecture impossible")) }
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(5_000), WorkbenchUiState())

    /** La teinte relevee ailleurs, que l'on nomme ici pour la poser sur la palette. */
    fun sample(colour: Rgb?) {
        sampled.value = colour
    }

    fun onIntent(intent: WorkbenchIntent) {
        when (intent) {
            is WorkbenchIntent.RecordMix -> viewModelScope.launch {
                val colour = sampled.value ?: return@launch
                mixesRepository.save(
                    PaletteMix(
                        name = intent.name,
                        hexColor = colour.toHex(),
                        recipe = intent.recipe,
                        dryingClass = intent.dryingClass,
                        mixedAt = clock.now(),
                        workshop = workshop(),
                    )
                )
            }

            is WorkbenchIntent.ForgetMix -> viewModelScope.launch {
                mixesRepository.delete(intent.id)
            }

            WorkbenchIntent.CleanSpentMixes -> viewModelScope.launch {
                mixes.spent(mixesRepository.all(), clock.now()).forEach { spent ->
                    spent.id?.let { mixesRepository.delete(it) }
                }
            }
        }
    }

    /** Le temps qui passe, qu'aucune ecriture en base ne signale. */
    private fun ticker(period: Duration) = flow {
        while (true) {
            emit(Unit)
            kotlinx.coroutines.delay(period)
        }
    }
}
