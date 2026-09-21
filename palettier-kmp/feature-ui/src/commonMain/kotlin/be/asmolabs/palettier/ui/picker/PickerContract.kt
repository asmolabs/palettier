package be.asmolabs.palettier.ui.picker

import be.asmolabs.palettier.domain.color.Rgb
import be.asmolabs.palettier.domain.mix.MixSuggestion
import be.asmolabs.palettier.domain.palette.Palette
import be.asmolabs.palettier.domain.project.DominantColour
import be.asmolabs.palettier.ui.SearchScope

/**
 * La photo affichee : ses octets d'origine, et ce qu'il faut pour placer un clic.
 *
 * <p>Classe ordinaire et non data class, volontairement. L'etat est un StateFlow, qui
 * compare chaque emission a la precedente ; une egalite structurelle comparerait les
 * millions d'octets de la photo a chaque frappe au clavier. Ici deux photos sont la meme
 * si c'est le meme objet, ce qui est exactement ce que l'ecran a besoin de savoir.</p>
 */
class LoadedImage(val bytes: ByteArray, val width: Int, val height: Int)

/**
 * Un groupe de releves : une partie du sujet, et les points qu'on y a pris.
 *
 * <p>C'est le chemin sans assistant. On nomme une partie -- la peau, la cape, le cuir des
 * sangles -- on y releve autant de points qu'on veut, et l'application en deduit le
 * degrade : le median tient la base, les plus sombres deviennent les ombres, les plus
 * clairs les lumieres. Rien n'est devine, chaque couche vient d'une mesure.</p>
 */
data class SampleGroup(val name: String, val samples: List<Rgb> = emptyList()) {

    /** Du sombre au clair : l'ordre dans lequel ces points deviendront des couches. */
    val ordered: List<Rgb> get() = samples.sortedBy { it.relativeLuminance() }
}

/** Le point releve, dans les coordonnees de l'image d'origine. */
data class PickPoint(val x: Int, val y: Int)

data class PickerUiState(
    val loading: Boolean = false,
    /** La photo a montrer, telle qu'elle a ete ouverte. */
    val image: LoadedImage? = null,
    /** Teintes relevees sur la photo, de la plus presente a la moins presente. */
    val dominant: List<DominantColour> = emptyList(),
    val selectedHex: String? = null,
    /** Ou le dernier releve a ete pris, pour le marquer sur la photo. */
    val pickedAt: PickPoint? = null,
    /** Rayon du carre moyenne autour du clic, en pixels de l'image. */
    val sampleRadius: Int = DEFAULT_RADIUS,
    /** Vrai quand le prochain clic designe le gris neutre au lieu de la teinte visee. */
    val pickingGrey: Boolean = false,
    /** Teinte du point choisi comme gris neutre, quand le peintre en a designe un. */
    val greyPointHex: String? = null,
    /** La teinte retenue une fois la dominante de la photo annulee. */
    val correctedHex: String? = null,
    /** Ou chercher : l'etagere, le catalogue entier, ou une palette. */
    val scope: SearchScope = SearchScope.Shelf,
    /** Les portees proposees, les palettes du peintre comprises. */
    val scopes: List<SearchScope> = listOf(SearchScope.Shelf, SearchScope.Catalogue),
    /** Vrai quand la portee choisie ne contient aucun tube : une palette vide, par exemple. */
    val emptyScope: Boolean = false,

    /**
     * Les melanges proposes pour atteindre la teinte relevee.
     *
     * <p>Un tube s'approche rarement d'une carnation ou d'une ombre : la reponse utile
     * est un melange, pas un nom de tube. Elle se demande, elle n'arrive pas toute seule
     * -- la recherche explore les dosages realisables et coute plusieurs centaines de
     * millisecondes sur le catalogue entier, ce qui n'a pas sa place derriere chaque clic
     * sur la photo.</p>
     */
    val suggestions: List<MixSuggestion> = emptyList(),
    val searching: Boolean = false,
    /** Nombre de tubes que la recherche s'autorise : a trois un melange se refait. */
    val maxPaints: Int = DEFAULT_MAX_PAINTS,
    val error: String? = null,

    /** Les groupes composes a partir des releves, dans l'ordre ou ils ont ete nommes. */
    val groups: List<SampleGroup> = emptyList(),
    /** Le groupe auquel le prochain releve sera ajoute. */
    val selectedGroup: String? = null,
    /** Les palettes disponibles pour etablir le plan du projet. */
    val palettes: List<Palette> = emptyList(),
    val projectPaletteId: Long? = null,
    /** Ce que la derniere action sur les groupes a donne : cree, refuse, enregistre. */
    val groupMessage: String? = null,
    val savingProject: Boolean = false,
) {
    /** Le groupe courant, quand il en reste un de ce nom. */
    val group: SampleGroup? get() = groups.firstOrNull { it.name == selectedGroup }

    /** Vrai des qu'un point a ete releve quelque part : sans cela, aucun plan a etablir. */
    val hasSamples: Boolean get() = groups.any { it.samples.isNotEmpty() }

    /** La teinte sur laquelle tout le reste travaille : corrigee si elle l'a ete. */
    val workingHex: String? get() = correctedHex ?: selectedHex

    companion object {
        /** Trois pixels de rayon : assez pour noyer le bruit, trop peu pour deborder. */
        const val DEFAULT_RADIUS = 3

        /** Trois tubes : au-dela, un melange vire au gris et ne se refait pas a l'identique. */
        const val DEFAULT_MAX_PAINTS = 3
    }
}

sealed interface PickerIntent {
    /** Une photo choisie par la plateforme : le domaine ne connait que ses octets. */
    data class Load(val bytes: ByteArray) : PickerIntent

    /**
     * Un clic sur la photo, dans les coordonnees de l'image d'origine.
     *
     * <p>L'ecran convertit lui-meme la position du clic, parce qu'il est le seul a savoir
     * a quel zoom il affiche. Le releve, lui, se fait toujours sur les pixels du fichier
     * ouvert : une image mise a l'echelle pour l'affichage a des couleurs interpolees, et
     * relever dessus reviendrait a mesurer le travail de la carte graphique.</p>
     */
    data class PickAt(val x: Int, val y: Int) : PickerIntent

    data class Select(val hex: String) : PickerIntent

    /** Designe une teinte censee etre grise, pour annuler la dominante de la photo. */
    data class UseAsGrey(val hex: String?) : PickerIntent

    /** Arme ou desarme le prochain clic pour designer le gris neutre. */
    data class PickGrey(val arming: Boolean) : PickerIntent

    data class SetRadius(val radius: Int) : PickerIntent

    /**
     * Change l'ensemble dans lequel chercher.
     *
     * <p>Une palette ne filtre pas un resultat deja calcule : elle change la question. Les
     * propositions sont donc reprises depuis le debut, sur les seuls tubes qu'elle
     * contient.</p>
     */
    data class ChooseScope(val scope: SearchScope) : PickerIntent

    /** Cherche avec quel melange faire la teinte retenue, dans la portee choisie. */
    data object FindMixes : PickerIntent

    data class SetMaxPaints(val count: Int) : PickerIntent

    // --- Les groupes de releves -------------------------------------------

    data class CreateGroup(val name: String) : PickerIntent

    data class SelectGroup(val name: String) : PickerIntent

    data class DeleteGroup(val name: String) : PickerIntent

    /** Ajoute la teinte retenue au groupe courant. */
    data object AddSampleToGroup : PickerIntent

    data class RemoveSample(val group: String, val hex: String) : PickerIntent

    data class ChooseProjectPalette(val id: Long) : PickerIntent

    /** Etablit le plan a partir des groupes, et l'enregistre comme projet. */
    data class CreateProject(val name: String) : PickerIntent
}
