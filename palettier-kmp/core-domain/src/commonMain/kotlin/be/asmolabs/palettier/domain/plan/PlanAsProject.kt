package be.asmolabs.palettier.domain.plan

import be.asmolabs.palettier.domain.palette.Palette
import be.asmolabs.palettier.domain.project.Project
import be.asmolabs.palettier.domain.project.ProjectLayer
import be.asmolabs.palettier.domain.project.ProjectZone

/**
 * Fige un plan en projet.
 *
 * <p>Un plan est un calcul : ses melanges se refont a chaque affichage, a partir des
 * tubes. Un projet est une decision : le decoupage en zones, la couleur visee de chaque
 * couche, la technique. Ce qui passe de l'un a l'autre, ce sont donc les decisions, pas
 * les dosages.</p>
 *
 * <p>Les tubes de la palette sont recopies dans le projet, et ce n'est pas une
 * duplication inutile : c'est ce qui rend un plan reproductible. Retirer un tube d'une
 * palette ne doit rien changer aux projets deja etablis.</p>
 *
 * <p>Les accents deviennent des couches de type ACCENT : leur role ne se devine pas au
 * nom -- "rougeur des pommettes" ne se reconnait par aucune convention -- et une echelle
 * de valeurs qui les avalerait placerait une variation locale entre deux marches.</p>
 */
fun PaintingPlan.asProject(name: String, palette: Palette?): Project = Project(
    name = name,
    subject = subject,
    approach = approach,
    palette = palette,
    paints = palette?.paints.orEmpty(),
    zones = zones.map { zone ->
        ProjectZone(
            name = zone.name,
            material = zone.material,
            note = zone.note,
            layers = zone.layers().map { it.asLayer(ProjectLayer.Kind.LADDER) } +
                zone.accents.map { it.asLayer(ProjectLayer.Kind.ACCENT) },
        )
    },
)

private fun PaintingPlan.Layer.asLayer(kind: ProjectLayer.Kind) = ProjectLayer(
    role = role,
    targetHex = target.toHex(),
    technique = technique,
    note = note,
    kind = kind,
)

/**
 * Un nom que la liste ne porte pas deja, suffixe au besoin.
 *
 * <p>Deux bustes du meme regiment s'appellent volontiers pareil, et refuser
 * l'enregistrement pour si peu ferait perdre le plan qu'on vient d'etablir. On numerote
 * plutot que d'opposer une erreur.</p>
 */
fun uniqueName(wanted: String, taken: Collection<String>): String {
    val base = wanted.trim().ifBlank { "Projet" }
    val used = taken.map { it.lowercase() }.toSet()

    if (base.lowercase() !in used) return base
    var suffix = 2
    while ("$base $suffix".lowercase() in used) suffix++
    return "$base $suffix"
}
