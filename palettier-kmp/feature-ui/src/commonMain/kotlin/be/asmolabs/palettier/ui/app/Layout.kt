package be.asmolabs.palettier.ui.app

import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

/**
 * A partir de quelle largeur deux colonnes valent mieux qu'une.
 *
 * <p>En dessous, les mettre cote a cote ne les rend pas plus lisibles : cela donne deux
 * colonnes trop etroites, et un nom de tube finit par se lire une lettre par ligne. Le
 * seuil vaut pour une fenetre de bureau retrecie autant que pour un telephone -- ce n'est
 * pas la plateforme qui decide, c'est la place.</p>
 */
val TWO_COLUMNS: Dp = 720.dp
