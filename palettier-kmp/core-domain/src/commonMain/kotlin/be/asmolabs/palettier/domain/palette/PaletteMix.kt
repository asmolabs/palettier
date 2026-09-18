package be.asmolabs.palettier.domain.palette

import be.asmolabs.palettier.domain.drying.DryingContext
import be.asmolabs.palettier.domain.drying.DryingTimeService
import be.asmolabs.palettier.domain.drying.Workshop
import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.paint.LayerThickness
import be.asmolabs.palettier.domain.paint.Medium
import kotlin.time.Clock
import kotlin.time.Duration
import kotlin.time.Instant

/**
 * Un melange pose sur la palette, avec l'heure a laquelle il l'a ete.
 *
 * <p>Le reste de l'application traite les melanges comme des calculs : on les refait a la
 * demande, ils n'ont pas d'existence propre. Celui-la en a une. A l'huile, un melange
 * reste travaillable des heures et survit parfois plusieurs jours sur une palette humide
 * -- c'est un fait physique, et personne ne se souvient au matin de ce qu'il a melange la
 * veille ni de quand.</p>
 */
data class PaletteMix(
    val id: Long? = null,
    /** Ce a quoi il sert : "gris rompu des ombres", "carnation de base". */
    val name: String,
    val hexColor: String,
    /** De quoi il est fait, en clair : c'est ce qui permet de le refaire. */
    val recipe: String = "",
    /** Vitesse du tube le plus lent du melange : c'est elle qui tient le temps ouvert. */
    val dryingClass: DryingClass = DryingClass.MEDIUM,
    val mixedAt: Instant = Clock.System.now(),
    /** Conditions figees a la preparation : c'est cet atelier-la qui commande. */
    val workshop: Workshop = Workshop.standard(),
)

/**
 * Ce qui est pose sur la palette, et pour combien de temps encore.
 *
 * <p>Une pate sur la palette n'est pas une couche sur la piece : elle n'est pas etalee,
 * elle forme un tas. Elle prend donc bien plus lentement, et c'est pour cela que le temps
 * ouvert se calcule ici avec une epaisseur chargee. Meme modele, autre forme.</p>
 */
class PaletteMixService(private val dryingTimeService: DryingTimeService = DryingTimeService()) {

    /**
     * Un melange et son etat.
     *
     * @param workable temps restant avant qu'il ne se travaille plus, nul s'il a pris
     * @param unusable temps restant avant qu'il ne serve plus a rien du tout
     */
    data class MixState(val mix: PaletteMix, val workable: Duration, val unusable: Duration) {
        /** Vrai tant qu'on peut encore s'en servir pour fondre et etaler. */
        val isOpen: Boolean get() = workable != Duration.ZERO

        /** Vrai quand il a pris et ne merite plus de place sur la palette. */
        val isSpent: Boolean get() = unusable == Duration.ZERO
    }

    fun states(mixes: List<PaletteMix>, now: Instant = Clock.System.now()): List<MixState> =
        mixes.map { state(it, now) }

    fun state(mix: PaletteMix, now: Instant = Clock.System.now()): MixState {
        val estimate = dryingTimeService.estimate(
            DryingContext(mix.dryingClass, Medium.NONE, 0.0, ON_THE_PALETTE, mix.workshop)
        )
        val age = (now - mix.mixedAt).coerceAtLeast(Duration.ZERO)
        return MixState(
            mix,
            (estimate.openTime - age).coerceAtLeast(Duration.ZERO),
            (estimate.touchDry - age).coerceAtLeast(Duration.ZERO),
        )
    }

    /** Ceux qui ont pris : la palette se nettoie aussi. */
    fun spent(mixes: List<PaletteMix>, now: Instant = Clock.System.now()): List<PaletteMix> =
        states(mixes, now).filter { it.isSpent }.map { it.mix }

    companion object {
        /** Un tas sur la palette, pas un film sur la piece. */
        private val ON_THE_PALETTE = LayerThickness.THICK
    }
}
