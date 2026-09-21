package be.asmolabs.palettier.ai

import kotlinx.serialization.Serializable

/**
 * Ce que l'on demande au modele de langage, et rien de plus : des zones, et pour chacune
 * des couleurs visees en hexadecimal.
 *
 * <p>Volontairement depourvu de tout dosage et de toute duree. Ces nombres-la sont
 * calcules ensuite a partir de la palette reelle, parce qu'un modele de langage les
 * inventerait avec aplomb.</p>
 */
@Serializable
data class PlanDraft(
    val approach: String = "",
    val zones: List<ZoneDraft> = emptyList(),
)

/**
 * Cinq couches nommees, plutot que deux listes.
 *
 * <p>Une liste imbriquee dans un objet imbrique dans une liste depasse les petits
 * modeles : ils produisent alors du JSON qui ne se referme pas. Des champs plats portent
 * la meme information, garantissent structurellement les deux ombres et les deux lumieres
 * demandees, et se remplissent correctement meme par un modele modeste.</p>
 */
@Serializable
data class ZoneDraft(
    val name: String = "",
    val material: String = "",
    val note: String = "",
    val base: LayerDraft? = null,
    val shadow1: LayerDraft? = null,
    val shadow2: LayerDraft? = null,
    val highlight1: LayerDraft? = null,
    val highlight2: LayerDraft? = null,
    val accent1: AccentDraft? = null,
    val accent2: AccentDraft? = null,
    val accent3: AccentDraft? = null,
)

/**
 * Une couleur qui n'appartient pas a l'echelle des valeurs.
 *
 * <p>Elle porte un nom, parce qu'elle se definit par l'endroit ou elle se pose et non par
 * son rang : "rougeur des pommettes" veut dire quelque chose, "couche 6" non.</p>
 */
@Serializable
data class AccentDraft(
    val name: String = "",
    val hex: String = "",
    val technique: String = "",
    val note: String = "",
)

@Serializable
data class LayerDraft(
    val hex: String = "",
    val technique: String = "",
    val note: String = "",
)
