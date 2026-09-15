package be.asmolabs.palettier.core.domain;

import be.asmolabs.palettier.core.color.Colorant;
import be.asmolabs.palettier.core.color.Rgb;
import java.util.Optional;

/**
 * Un pigment du Colour Index, avec les proprietes qui en decoulent.
 *
 * <p>C'est la piece centrale du catalogue : la vitesse de sechage et le pouvoir
 * colorant d'une huile ne sont pas des choix de marque, ce sont des proprietes du
 * pigment. Une terre d'ombre (PBr7) seche vite chez tout le monde, un cadmium (PY35)
 * seche lentement chez tout le monde. Les stocker une fois ici evite de les
 * re-deviner pour chaque tube de chaque gamme.</p>
 *
 * @param code            code Colour Index, par exemple PBr7 ou PW6
 * @param name            nom usuel du pigment
 * @param color           teinte en masse ton, indicative
 * @param opacity         pouvoir couvrant du pigment pur
 * @param dryingClass     vitesse de sechage a l'huile
 * @param tintingStrength pouvoir colorant relatif, entre 0,05 et 1
 * @param tint            couleur du pigment coupe de blanc, ou {@code null} si elle n'a
 *                        pas ete relevee. Quand elle l'est, le melange passe au modele
 *                        de Kubelka-Munk a deux constantes, seul capable de representer
 *                        un pigment dont le ton de masse et la teinte diluee divergent.
 */
public record Pigment(String code,
                      String name,
                      Rgb color,
                      Opacity opacity,
                      DryingClass dryingClass,
                      double tintingStrength,
                      Rgb tint) {

    public Optional<Rgb> tintColor() {
        return Optional.ofNullable(tint);
    }

    /** Constantes de melange du pigment, a deux constantes si sa teinte diluee est connue. */
    public Colorant colorant() {
        return tint == null
                ? Colorant.ofMasstone(color)
                : Colorant.ofMasstoneAndTint(color, tint, Colorant.REFERENCE_TINT_CONCENTRATION);
    }

    /** Pigment de repli, utilise quand un code n'est pas encore reference. */
    public static Pigment unknown(String code) {
        return new Pigment(code, "Pigment non reference", new Rgb(0.5, 0.5, 0.5),
                Opacity.SEMI_OPAQUE, DryingClass.MEDIUM, 0.5, null);
    }
}
