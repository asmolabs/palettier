package be.asmolabs.palettier.core.service;

import be.asmolabs.palettier.core.color.Rgb;
import be.asmolabs.palettier.core.domain.DryingClass;
import be.asmolabs.palettier.core.domain.OilPaint;
import java.util.List;
import java.util.Set;

/** Types de valeur echanges autour du melange de couleurs. */
public final class MixModels {

    private MixModels() {
    }

    /**
     * Une huile et sa dose dans le melange, exprimee en parts (gouttes, pointes de pinceau,
     * peu importe l'unite tant qu'elle est la meme pour tout le melange).
     */
    public record PaintPart(OilPaint paint, double parts) {
        public PaintPart {
            if (parts <= 0) {
                throw new IllegalArgumentException("La dose doit etre strictement positive");
            }
        }

        public static PaintPart of(OilPaint paint, double parts) {
            return new PaintPart(paint, parts);
        }
    }

    /**
     * Contribution d'une huile au melange.
     *
     * @param volumeShare  part en volume, entre 0 et 1
     * @param pigmentShare part dans la couleur finale une fois le pouvoir colorant pris en compte
     */
    public record MixComponent(OilPaint paint, double parts, double volumeShare, double pigmentShare) {
    }

    /** Resultat complet d'un melange. */
    public record MixResult(Rgb color,
                            List<MixComponent> components,
                            DryingClass dryingClass,
                            Set<String> pigments,
                            List<String> warnings) {

        public String hex() {
            return color.toHex();
        }
    }

    /** Une proposition de melange pour approcher une couleur cible. */
    public record MixSuggestion(List<PaintPart> parts, Rgb color, double deltaE) {

        /** Description lisible du type "3 parts de X + 1 part de Y". */
        public String describe() {
            return parts.stream()
                    .map(p -> "%s part%s de %s".formatted(
                            trim(p.parts()), p.parts() > 1 ? "s" : "", p.paint().displayName()))
                    .reduce((a, b) -> a + "  +  " + b)
                    .orElse("");
        }

        private static String trim(double value) {
            return value == Math.rint(value) ? String.valueOf((long) value) : String.valueOf(value);
        }
    }

    /** Une huile du catalogue et son ecart a une couleur cible. */
    public record PaintMatch(OilPaint paint, double deltaE) {

        /** Traduction du delta E en langage de peintre. */
        public String verdict() {
            return switch ((int) Math.floor(deltaE)) {
                case 0, 1 -> "Identique a l'oeil";
                case 2, 3 -> "Tres proche";
                case 4, 5, 6, 7 -> "Proche, ecart visible cote a cote";
                case 8, 9, 10, 11, 12, 13, 14 -> "Meme famille de teinte";
                default -> "Couleur differente";
            };
        }
    }
}
