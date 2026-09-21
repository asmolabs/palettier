package be.asmolabs.palettier.ui

import be.asmolabs.palettier.domain.paint.Paint
import be.asmolabs.palettier.domain.port.PaintCatalogRepository
import be.asmolabs.palettier.domain.port.PaletteRepository

/**
 * L'ensemble de tubes dans lequel chercher.
 *
 * <p>Ce n'est pas un filtre d'affichage, c'est la difference entre une reponse theorique
 * et une recette faisable. Chercher une couleur dans les six cent quatre-vingts huiles du
 * catalogue donne un melange souvent compose de tubes qu'on ne possede pas ; la meme
 * recherche limitee a la palette sortie sur la table donne quelque chose a faire tout de
 * suite.</p>
 *
 * <p>C'est aussi ce qui donne aux palettes leur second role, celui que l'application
 * revendique : une palette n'est pas qu'un rangement, c'est un filtre de recherche.</p>
 */
sealed interface SearchScope {

    /** Comment l'appeler a l'ecran. */
    val label: String

    /** Les six cent quatre-vingts huiles : la reponse la plus juste, pas la plus utile. */
    data object Catalogue : SearchScope {
        override val label = "Tout le catalogue"
    }

    /** Ce qui est reellement sur l'etagere. */
    data object Shelf : SearchScope {
        override val label = "Ce que je possede"
    }

    /** Une palette nommee : ce qui est sorti sur la table pour cette piece. */
    data class OfPalette(val id: Long, val name: String) : SearchScope {
        override val label = name
    }
}

/**
 * Les tubes que cette portee designe.
 *
 * <p>Une palette effacee entre deux ouvertures de l'ecran rend une liste vide plutot
 * qu'une erreur : c'est a l'ecran de le dire, pas au chargement de tomber.</p>
 */
suspend fun SearchScope.candidates(
    catalog: PaintCatalogRepository,
    palettes: PaletteRepository,
): List<Paint> = when (this) {
    SearchScope.Catalogue -> catalog.all()
    SearchScope.Shelf -> catalog.inStock()
    is SearchScope.OfPalette -> palettes.findById(id)?.paints.orEmpty()
}

/** Les portees a proposer : les deux constantes, puis les palettes du peintre. */
fun scopesOf(palettes: List<be.asmolabs.palettier.domain.palette.Palette>): List<SearchScope> =
    listOf(SearchScope.Shelf, SearchScope.Catalogue) +
        palettes.mapNotNull { palette -> palette.id?.let { SearchScope.OfPalette(it, palette.name) } }
