package be.asmolabs.palettier.ui.app

/**
 * Les sections, dans l'ordre ou elles se presentent.
 *
 * <p>Partagees par les deux clients, parce que l'ordre est une decision de l'application
 * et non de la plateforme : Aujourd'hui d'abord, c'est la question du matin. Ce qui
 * differe, c'est la facon de naviguer -- une colonne sur le bureau, un tiroir sur un
 * telephone -- pas ce vers quoi l'on navigue.</p>
 *
 * <p>Le raccourci est declare par la section et non deduit de son rang : inserer une
 * section en tete ne doit pas deplacer tout ce qui suit sous les doigts de quelqu'un qui
 * a pris ses habitudes. Il ne sert que la ou il y a un clavier.</p>
 */
enum class Section(val title: String, val shortcut: Int) {
    Today("Aujourd'hui", 0),
    Projects("Projets", 1),
    Palettes("Palettes", 2),
    Picker("Pipette", 3),
    Assistant("Assistant", 4),
    Mixer("Melangeur", 5),
    Catalogue("Catalogue", 6),
    Drying("Sechage", 7),
    Recipes("Recettes", 8),
    Settings("Parametres", 9),
}
