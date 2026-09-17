package be.asmolabs.palettier.ui;

import javafx.scene.Node;

/**
 * Une section de l'application, representee par une entree dans la barre laterale.
 * Chaque implementation est un bean Spring ; la fenetre principale collecte toutes
 * celles qui sont declarees, dans l'ordre donne par {@link #order()}.
 */
public interface AppView {

    /** Libelle dans la navigation. */
    String title();

    /** Phrase affichee sous le titre de la page : ce que la section permet de faire. */
    String subtitle();

    /**
     * Construit le contenu de la section. Appele une seule fois, a la premiere
     * ouverture, sur le fil d'affichage JavaFX.
     */
    Node create();

    int order();

    /**
     * Chiffre du raccourci clavier, de 0 a 9, ou -1 pour aucun.
     *
     * <p>Declare par la section plutot que deduit de son rang. Une section inseree en
     * tete decalerait sinon tous les raccourcis d'un cran, et le peintre qui tape
     * machinalement sur 3 pour la pipette tomberait ailleurs. L'ordre d'affichage se
     * reorganise librement, les raccourcis ne bougent pas.</p>
     */
    default int shortcut() {
        return -1;
    }

    /**
     * Vrai pour detacher cette section de celles qui precedent par un filet.
     *
     * <p>Sert a separer ce qui releve du travail courant de ce qui n'en releve pas :
     * on ne va pas dans les reglages en peignant.</p>
     */
    default boolean separatorBefore() {
        return false;
    }
}
