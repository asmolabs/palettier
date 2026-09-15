package be.asmolabs.palettier.core.catalog;

import static org.assertj.core.api.Assertions.assertThat;

import be.asmolabs.palettier.core.color.Rgb;
import be.asmolabs.palettier.core.domain.DryingClass;
import be.asmolabs.palettier.core.domain.OilPaint;
import be.asmolabs.palettier.core.domain.Opacity;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Set;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

/**
 * Verifie les fichiers du catalogue eux-memes. Ce sont des donnees editees a la main :
 * ces tests sont le garde-fou qui attrape une faute de frappe avant le demarrage.
 */
class CatalogDataTest {

    /** Les seules gammes attendues : toutes a l'huile, aucune acrylique. */
    private static final Set<String> EXPECTED_BRANDS = Set.of(
            "Winsor & Newton",
            "Winton",
            "Schmincke Norma",
            "Schmincke Norma Blue",
            "Gamblin",
            "Williamsburg",
            "Scale75 Floww",
            "Abteilung 502");

    private final PigmentIndex pigments = new PigmentIndex();
    private final List<OilPaint> catalog = new CatalogLoader(pigments).load();

    @Test
    @DisplayName("le catalogue ne contient que les gammes a l'huile retenues")
    void onlyExpectedBrands() {
        Set<String> brands = new LinkedHashSet<>(catalog.stream().map(OilPaint::getBrand).toList());

        assertThat(brands).containsExactlyInAnyOrderElementsOf(EXPECTED_BRANDS);
    }

    @Test
    @DisplayName("chaque pigment cite par une gamme existe dans la table des pigments")
    void everyCitedPigmentIsReferenced() {
        Set<String> cited = new LinkedHashSet<>();
        catalog.forEach(paint -> cited.addAll(paint.getPigments()));

        assertThat(pigments.unknownAmong(cited))
                .as("pigments a ajouter dans catalog/pigments.csv")
                .isEmpty();
    }

    @Test
    @DisplayName("chaque tube est exploitable : teinte valide, pigment declare, pouvoir colorant sense")
    void everyPaintIsUsable() {
        assertThat(catalog).isNotEmpty();
        assertThat(catalog).allSatisfy(paint -> {
            assertThat(paint.getName()).isNotBlank();
            assertThat(paint.getHexColor()).matches("#[0-9A-F]{6}");
            assertThat(paint.getPigments()).isNotEmpty();
            assertThat(paint.getTintingStrength()).isBetween(0.05, 1.0);
            assertThat(paint.getOpacity()).isNotNull();
            assertThat(paint.getDryingClass()).isNotNull();
        });
    }

    @Test
    @DisplayName("un tube n'apparait pas deux fois dans la meme gamme")
    void noDuplicateWithinABrand() {
        List<String> keys = catalog.stream().map(p -> p.getBrand() + "|" + p.getName()).toList();

        assertThat(keys).doesNotHaveDuplicates();
    }

    @Test
    @DisplayName("le sechage est deduit du pigment le plus lent")
    void dryingComesFromTheSlowestPigment() {
        // PBr7 seche vite, PY35 (cadmium) tres lentement : le melange suit le cadmium.
        assertThat(pigments.dryingClassFor(Set.of("PBr7"))).isEqualTo(DryingClass.FAST);
        assertThat(pigments.dryingClassFor(Set.of("PBr7", "PY35"))).isEqualTo(DryingClass.VERY_SLOW);
    }

    @Test
    @DisplayName("l'opacite retenue est celle du pigment le plus couvrant")
    void opacityComesFromTheMostCoveringPigment() {
        assertThat(pigments.opacityFor(Set.of("PB29"))).isEqualTo(Opacity.TRANSPARENT);
        assertThat(pigments.opacityFor(Set.of("PB29", "PW6"))).isEqualTo(Opacity.OPAQUE);
    }

    @Test
    @DisplayName("un tube dont la teinte n'est pas publiee est marque comme deduit")
    void paintsWithoutAPublishedColourAreFlagged() {
        // Schmincke ne publie pas de valeur numerique pour Norma Blue : ces teintes sont
        // deduites des pigments, et l'application doit le dire plutot que de faire croire
        // a un releve.
        assertThat(catalog).filteredOn(paint -> paint.getBrand().equals("Schmincke Norma Blue"))
                .isNotEmpty()
                .allSatisfy(paint -> assertThat(paint.isColorDerived()).isTrue());

        // Le drapeau doit suivre le fichier, gamme par gamme : W&N ne publie aucune valeur
        // colorimetrique, mais le catalogue en conserve pour la plupart de ses teintes.
        // Celles qui n'en ont pas doivent se declarer deduites, pas se faire passer pour
        // relevees.
        assertThat(paintNamed("Winsor & Newton", "Titanium White").isColorDerived())
                .as("teinte presente dans le fichier")
                .isFalse();
        assertThat(paintNamed("Winsor & Newton", "Oriental Blue").isColorDerived())
                .as("teinte absente du fichier, donc deduite des pigments")
                .isTrue();
    }

    @Test
    @DisplayName("a defaut de teinte declaree, le melange des pigments en fournit une")
    void pigmentMixProvidesAFallbackColour() {
        Rgb white = pigments.colorFor(Set.of("PW6"));
        Rgb greyed = pigments.colorFor(Set.of("PW6", "PBk11"));

        assertThat(greyed.relativeLuminance()).isLessThan(white.relativeLuminance());
    }

    @Test
    @DisplayName("les tubes Abteilung renumerotes gardent leur ancienne reference")
    void renumberedPaintsKeepTheirOldReference() {
        // AK a renumerote sa gamme : ABT004 est devenu AKABT004. Le peintre a les deux
        // etiquettes sur son etagere, il doit retrouver le tube par l'une ou l'autre.
        assertThat(paintNamed("Abteilung 502", "Bitume"))
                .satisfies(paint -> {
                    assertThat(paint.getCode()).isEqualTo("AKABT004");
                    assertThat(paint.getLegacyCode()).isEqualTo("ABT004");
                });

        // Les 12 couleurs apparues avec la nouvelle numerotation n'ont pas d'equivalent.
        assertThat(paintNamed("Abteilung 502", "Vermilion").getLegacyCode()).isEmpty();

        // Les couleurs retirees de la gamme n'existent plus que sous l'ancienne reference.
        assertThat(paintNamed("Abteilung 502", "Gundam Blue"))
                .satisfies(paint -> {
                    assertThat(paint.getCode()).isEmpty();
                    assertThat(paint.getLegacyCode()).isEqualTo("ABT500");
                    assertThat(paint.isPigmentsVerified())
                            .as("fiche disparue du site : pigments deduits du nom")
                            .isFalse();
                });

        // Toute reference ancienne est celle de la nouvelle, sans le prefixe du fabricant.
        assertThat(catalog).filteredOn(paint -> paint.getBrand().equals("Abteilung 502")
                        && !paint.getCode().isBlank() && !paint.getLegacyCode().isBlank())
                .allSatisfy(paint -> assertThat(paint.getCode()).isEqualTo("AK" + paint.getLegacyCode()));
    }

    private OilPaint paintNamed(String brand, String name) {
        return catalog.stream()
                .filter(paint -> paint.getBrand().equals(brand) && paint.getName().equals(name))
                .findFirst()
                .orElseThrow(() -> new AssertionError("tube absent du catalogue : " + brand + " - " + name));
    }
}
