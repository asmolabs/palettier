package be.asmolabs.palettier.ai;

import java.util.List;

/**
 * Ce que l'on demande au modele de langage, et rien de plus : des zones, et pour chacune
 * trois couleurs visees en hexadecimal.
 *
 * <p>Volontairement depourvu de tout dosage et de toute duree. Ces nombres-la sont
 * calcules ensuite a partir de la palette reelle, parce qu'un modele de langage les
 * inventerait avec aplomb.</p>
 */
record PlanDraft(String approach, List<ZoneDraft> zones) {

    /**
     * Cinq couches nommees, plutot que deux listes.
     *
     * <p>Une liste imbriquee dans un objet imbrique dans une liste depasse les petits
     * modeles : ils produisent alors du JSON qui ne se referme pas. Des champs plats
     * portent la meme information, garantissent structurellement les deux ombres et les
     * deux lumieres demandees, et se remplissent correctement meme par un modele
     * modeste.</p>
     *
     * @param shadow1    ombre legere, dans les demi-tons
     * @param shadow2    ombre profonde, dans les creux fermes
     * @param highlight1 premier eclairci, sur les volumes exposes
     * @param highlight2 point lumineux, sur une arete seulement
     * @param accent1    variation locale, facultative
     * @param accent2    variation locale, facultative
     * @param accent3    variation locale, facultative
     */
    record ZoneDraft(String name, String material, String note,
                     LayerDraft base,
                     LayerDraft shadow1, LayerDraft shadow2,
                     LayerDraft highlight1, LayerDraft highlight2,
                     AccentDraft accent1, AccentDraft accent2, AccentDraft accent3) {
    }

    /**
     * Une couleur qui n'appartient pas a l'echelle des valeurs.
     *
     * <p>Elle porte un nom, parce qu'elle se definit par l'endroit ou elle se pose et non
     * par son rang : "rougeur des pommettes" veut dire quelque chose, "couche 6" non.</p>
     *
     * @param name ou la poser, en quelques mots
     */
    record AccentDraft(String name, String hex, String technique, String note) {
    }

    /**
     * @param hex       couleur visee au format #RRGGBB
     * @param technique nom de la technique a l'huile employee pour cette couche
     */
    record LayerDraft(String hex, String technique, String note) {
    }
}
