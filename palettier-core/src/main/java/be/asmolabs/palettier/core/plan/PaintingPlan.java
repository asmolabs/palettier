package be.asmolabs.palettier.core.plan;

import be.asmolabs.palettier.core.color.Rgb;
import be.asmolabs.palettier.core.service.MixModels.MixSuggestion;
import java.util.List;

/**
 * Plan de peinture par zones : pour chaque partie du sujet, une teinte de base, une
 * ombre et une lumiere, chacune accompagnee du melange a faire avec la palette choisie.
 *
 * <p>Repartition des roles : les couleurs a viser relevent du jugement -- celui de
 * l'assistant, ou le votre ; le melange qui y conduit est calcule par {@code MixSearch}.
 * L'ecart affiche est donc une mesure, pas une affirmation.</p>
 *
 * <p>Ce type vit dans le module metier et non dans celui de l'assistant : un plan de
 * peinture n'a rien qui tienne a la facon dont il a ete obtenu. Un projet enregistre en
 * produit un tout aussi bien.</p>
 */
public record PaintingPlan(String subject,
                           String paletteName,
                           String approach,
                           List<Zone> zones) {

    /**
     * Une partie du sujet : le visage, la cape, le cuir des sangles...
     *
     * <p>Deux ombres et deux lumieres au minimum. Une seule de chaque suffit a poser un
     * volume, jamais a le rendre : sur une figurine, la profondeur nait de l'etagement
     * des valeurs, et c'est le deuxieme ton qui fait la difference entre une piece plate
     * et une piece modelee.</p>
     *
     * @param shadows    de la plus legere a la plus profonde
     * @param highlights du premier eclairci au point lumineux
     */
    public record Zone(String name, String material, String note,
                       Layer base, List<Layer> shadows, List<Layer> highlights) {

        /** Toutes les couches, de la plus sombre a la plus claire : l'ordre de lecture d'un degrade. */
        public List<Layer> layers() {
            List<Layer> ordered = new java.util.ArrayList<>(shadows.reversed());
            ordered.add(base);
            ordered.addAll(highlights);
            return List.copyOf(ordered);
        }
    }

    /**
     * Une couche a poser.
     *
     * @param target    couleur visee, choisie par le modele
     * @param recipe    melange calcule pour l'atteindre avec la palette, ou {@code null}
     *                  si la palette est vide
     * @param achieved  couleur reellement obtenue par ce melange
     * @param deltaE    ecart mesure entre la couleur visee et celle obtenue
     */
    public record Layer(String role, Rgb target, String technique, String note,
                        MixSuggestion recipe, Rgb achieved, double deltaE) {

        /** Ce que la palette permet reellement d'atteindre pour cette couche. */
        public String reachability() {
            if (recipe == null) {
                return "palette vide";
            }
            if (deltaE < 2) {
                return "atteignable tel quel";
            }
            if (deltaE < 5) {
                return "tres proche, ecart invisible sur la piece";
            }
            if (deltaE < 12) {
                return "approche, a rattraper au glacis";
            }
            return "hors de portee de cette palette";
        }
    }
}
