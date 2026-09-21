package be.asmolabs.palettier.domain.plan

import be.asmolabs.palettier.domain.color.Rgb
import be.asmolabs.palettier.domain.color.deltaE2000
import be.asmolabs.palettier.domain.mix.ColorMixService
import be.asmolabs.palettier.domain.paint.Paint
import be.asmolabs.palettier.domain.paint.Technique
import be.asmolabs.palettier.domain.palette.Palette

/**
 * Batit un plan de peinture a partir de couleurs relevees sur une reference.
 *
 * <p>C'est le chemin sans assistant : on photographie ce qu'on veut obtenir, on preleve
 * plusieurs points par partie du sujet -- le creux de l'orbite, la joue, l'arete du nez --
 * et l'application en tire le plan. Rien n'est devine : chaque couche vient d'une
 * mesure.</p>
 *
 * <p>Les roles sont deduits de la clarte des points, et non de l'ordre de prelevement :
 * on ne releve pas une reference dans l'ordre, on releve ce qu'on voit. Le point median
 * devient la base, les plus sombres les ombres, les plus clairs les lumieres.</p>
 */
class SampledPlanService(private val mixer: ColorMixService) {

    /**
     * Un groupe de points releves sur une meme partie du sujet.
     *
     * @param name     ce que la partie represente : "peau", "cape", "cuir des sangles"
     * @param material precision libre sur la matiere, eventuellement vide
     * @param samples  couleurs relevees, dans n'importe quel ordre
     */
    data class ColourGroup(
        val name: String,
        val material: String = "",
        val samples: List<Rgb> = emptyList(),
    )

    /**
     * @param maxPaints nombre maximal de tubes par melange
     * @throws IllegalArgumentException si aucun groupe ne porte de releve
     */
    suspend fun plan(
        subject: String,
        palette: Palette?,
        groups: List<ColourGroup>,
        maxPaints: Int,
    ): PaintingPlan {
        val usable = groups.filter { it.samples.isNotEmpty() }
        require(usable.isNotEmpty()) {
            "Aucun point releve : ajoutez au moins une couleur a un groupe."
        }

        return PaintingPlan(
            subject = subject,
            paletteName = palette?.name ?: "sans palette",
            approach = "Plan etabli a partir de ${usable.sumOf { it.samples.size }} points " +
                "releves sur la reference.",
            zones = usable.map { zone(it, palette, maxPaints) },
        )
    }

    private suspend fun zone(group: ColourGroup, palette: Palette?, maxPaints: Int): PaintingPlan.Zone {
        val ordered = group.samples.sortedBy { it.relativeLuminance() }

        // Le point median tient la valeur locale : c'est la base. Ce qui est plus sombre
        // descend dans les ombres, ce qui est plus clair monte dans les lumieres.
        val baseIndex = ordered.size / 2

        val shadows = (baseIndex - 1 downTo 0).mapIndexed { rank, index ->
            layer("Ombre ${rank + 1}", ordered[index], Technique.GLAZE, palette, maxPaints)
        }
        val highlights = (baseIndex + 1 until ordered.size).mapIndexed { rank, index ->
            layer("Lumiere ${rank + 1}", ordered[index], Technique.HIGHLIGHT, palette, maxPaints)
        }

        return PaintingPlan.Zone(
            name = group.name,
            material = group.material,
            note = noteFor(ordered.size),
            base = layer("Base", ordered[baseIndex], Technique.BASE_LAYER, palette, maxPaints),
            shadows = shadows,
            highlights = highlights,
        )
    }

    /** Un mot sur ce que le nombre de points permet, ou ne permet pas. */
    private fun noteFor(sampleCount: Int): String = when (sampleCount) {
        1 -> "Un seul point releve : la valeur locale, sans modele. Relevez une ombre et " +
            "une lumiere pour obtenir un degrade."
        2 -> "Deux points : une base et un ecart. Un troisieme de l'autre cote donnerait " +
            "un vrai degrade."
        else -> "$sampleCount points releves sur la reference."
    }

    /**
     * La couche visant cette couleur, avec le melange qui y conduit.
     *
     * <p>Sans palette, ou quand aucune recette n'approche, la couche garde sa couleur
     * visee et le dit par un ecart nul faute de melange : un plan incomplet vaut mieux
     * qu'un plan refuse.</p>
     */
    private suspend fun layer(
        role: String,
        target: Rgb,
        technique: Technique,
        palette: Palette?,
        maxPaints: Int,
    ): PaintingPlan.Layer {
        val paints: List<Paint> = palette?.paints.orEmpty()
        val best = if (paints.isEmpty()) {
            null
        } else {
            mixer.suggestMixes(target, paints, maxResults = 1, maxPaints = maxPaints).firstOrNull()
        }

        return if (best == null) {
            PaintingPlan.Layer(role, target, technique.label, "", null, target, 0.0)
        } else {
            PaintingPlan.Layer(
                role, target, technique.label, "",
                best, best.color, deltaE2000(target, best.color),
            )
        }
    }
}
