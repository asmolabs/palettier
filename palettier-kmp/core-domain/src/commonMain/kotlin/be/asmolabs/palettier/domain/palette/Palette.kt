package be.asmolabs.palettier.domain.palette

import be.asmolabs.palettier.domain.paint.DryingClass
import be.asmolabs.palettier.domain.paint.Paint

/**
 * Une selection nommee de tubes : la palette qu'on prepare pour un sujet donne.
 *
 * <p>Sur une figurine, on ne travaille jamais avec le catalogue entier mais avec six a
 * douze tubes choisis ensemble. La palette sert donc autant a s'organiser qu'a
 * restreindre les recherches : chercher un melange "dans ma palette" donne une reponse
 * utilisable, la meme recherche sur sept cents tubes donne une reponse theorique.</p>
 */
data class Palette(
    val id: Long? = null,
    val name: String,
    /** Ce a quoi la palette est destinee : "carnations 1/10", "blindage vert olive"... */
    val purpose: String = "",
    val notes: String = "",
    val paints: List<Paint> = emptyList(),
) {
    /**
     * Classe de sechage imposee par la palette : celle de son tube le plus lent. C'est
     * elle qui commande le planning d'une seance menee avec ces couleurs.
     */
    fun slowestDryingClass(): DryingClass =
        paints.fold(DryingClass.FAST) { slowest, paint -> slowest.slowest(paint.dryingClass) }

    /** Pigments distincts presents dans la palette : au-dela de six, les melanges grisent. */
    fun pigments(): Set<String> = paints.flatMapTo(LinkedHashSet()) { it.pigments }

    /** Ajoute un tube s'il n'y est pas deja. */
    fun withPaint(paint: Paint): Palette =
        if (paints.any { it.id != null && it.id == paint.id }) this else copy(paints = paints + paint)

    fun withoutPaint(paint: Paint): Palette =
        copy(paints = paints.filterNot { it.id != null && it.id == paint.id })
}
