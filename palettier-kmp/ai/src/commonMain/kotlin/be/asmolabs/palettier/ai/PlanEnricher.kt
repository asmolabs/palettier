package be.asmolabs.palettier.ai

import be.asmolabs.palettier.domain.color.Rgb
import be.asmolabs.palettier.domain.color.deltaE2000
import be.asmolabs.palettier.domain.color.mix
import be.asmolabs.palettier.domain.mix.ColorMixService
import be.asmolabs.palettier.domain.palette.Palette
import be.asmolabs.palettier.domain.plan.PaintingPlan

/**
 * Transforme la reponse du modele en plan utilisable.
 *
 * <p>C'est ici que se joue la repartition des roles : le modele choisit quelles couleurs
 * viser -- un jugement de peintre -- et l'application calcule comment les atteindre avec
 * les tubes reellement disponibles. L'ecart affiche est donc une mesure, pas une
 * affirmation.</p>
 */
class PlanEnricher(private val mixer: ColorMixService) {

    suspend fun enrich(subject: String, palette: Palette, draft: PlanDraft, maxPaints: Int): PaintingPlan {
        val zones = draft.zones.map { zone ->
            val base = layer("Base", zone.base, palette, maxPaints)
            PaintingPlan.Zone(
                name = zone.name,
                material = zone.material,
                note = zone.note,
                base = base,
                shadows = listOf(
                    step("Ombre 1", zone.shadow1, base, zone.shadow2, palette, maxPaints),
                    step("Ombre 2", zone.shadow2, base, zone.shadow1, palette, maxPaints),
                ),
                highlights = listOf(
                    step("Lumiere 1", zone.highlight1, base, zone.highlight2, palette, maxPaints),
                    step("Lumiere 2", zone.highlight2, base, zone.highlight1, palette, maxPaints),
                ),
                accents = accents(zone, palette, maxPaints),
            )
        }
        return PaintingPlan(subject, palette.name, draft.approach, zones)
    }

    /**
     * Une couche du degrade, avec un repli si le modele a saute un champ.
     *
     * <p>Plutot que de renoncer, on interpole : la couche manquante devient le melange a
     * parts egales de la base et de l'autre couche du meme cote. Ce n'est pas un jugement
     * invente, c'est le milieu de deux couleurs que le modele a lui-meme choisies.</p>
     */
    private suspend fun step(
        role: String,
        draft: LayerDraft?,
        base: PaintingPlan.Layer,
        sibling: LayerDraft?,
        palette: Palette,
        maxPaints: Int,
    ): PaintingPlan.Layer {
        if (draft != null && draft.hex.isNotBlank()) return layer(role, draft, palette, maxPaints)

        if (sibling == null || sibling.hex.isBlank()) {
            return layer(
                role,
                LayerDraft(base.target.toHex(), "", "Couche absente de la reponse."),
                palette, maxPaints,
            )
        }

        val between = mix(listOf(base.target, parse(sibling.hex)), listOf(1.0, 1.0))
        return layer(
            role,
            LayerDraft(
                between.toHex(), sibling.technique,
                "Ton interpole entre la base et l'autre couche : absent de la reponse du modele.",
            ),
            palette, maxPaints,
        )
    }

    /**
     * Les variations locales effectivement proposees.
     *
     * <p>Contrairement aux couches de l'echelle, on ne comble pas les manquantes : une
     * variation locale est une observation, elle ne s'interpole pas.</p>
     */
    private suspend fun accents(zone: ZoneDraft, palette: Palette, maxPaints: Int): List<PaintingPlan.Layer> {
        val found = mutableListOf<PaintingPlan.Layer>()
        listOfNotNull(zone.accent1, zone.accent2, zone.accent3)
            .filter { it.hex.isNotBlank() }
            .forEach { accent ->
                val name = accent.name.ifBlank { "Variation ${found.size + 1}" }
                found += layer(name, LayerDraft(accent.hex, accent.technique, accent.note), palette, maxPaints)
            }
        return found.toList()
    }

    private suspend fun layer(
        role: String,
        draft: LayerDraft?,
        palette: Palette,
        maxPaints: Int,
    ): PaintingPlan.Layer {
        val target = parse(draft?.hex)
        val technique = draft?.technique.orEmpty()
        val note = draft?.note.orEmpty()

        val best = mixer.suggestMixes(target, palette.paints, maxResults = 1, maxPaints = maxPaints)
            .firstOrNull()
            ?: return PaintingPlan.Layer(role, target, technique, note, null, target, 0.0)

        return PaintingPlan.Layer(role, target, technique, note, best, best.color, deltaE2000(target, best.color))
    }

    companion object {
        /** Repli quand le modele se trompe de format : un gris moyen, visible et neutre. */
        private val FALLBACK = Rgb(0.5, 0.5, 0.5)

        /** Un modele de langage se trompe parfois de format : on n'en fait pas un incident. */
        fun parse(hex: String?): Rgb = runCatching { Rgb.ofHex(hex ?: "") }.getOrDefault(FALLBACK)
    }
}
