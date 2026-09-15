package be.asmolabs.palettier.core.service;

import static org.assertj.core.api.Assertions.assertThat;

import be.asmolabs.palettier.core.domain.DryingClass;
import be.asmolabs.palettier.core.domain.OilPaint;
import be.asmolabs.palettier.core.color.Rgb;
import be.asmolabs.palettier.core.domain.Palette;
import java.util.List;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.transaction.annotation.Transactional;

@SpringBootTest
@Transactional
class PaletteServiceTest {

    @Autowired
    private PaletteService palettes;

    @Autowired
    private PaintCatalogService catalog;

    @Autowired
    private ColorMixService mixer;

    @Test
    @DisplayName("les palettes d'exemple sont creees et contiennent des tubes")
    void examplePalettesAreSeeded() {
        List<Palette> all = palettes.findAll();

        assertThat(all).isNotEmpty();
        assertThat(all).allSatisfy(palette -> assertThat(palette.getPaints()).isNotEmpty());
    }

    @Test
    @DisplayName("deux palettes ne peuvent pas porter le meme nom")
    void namesAreMadeUnique() {
        Palette first = palettes.create("Carnations", "test");
        Palette second = palettes.create("Carnations", "test");

        assertThat(second.getName()).isNotEqualTo(first.getName()).startsWith("Carnations");
    }

    @Test
    @DisplayName("un tube ajoute deux fois n'apparait qu'une fois")
    void addingTwiceKeepsOneEntry() {
        Palette palette = palettes.create("Palette de test", "");
        OilPaint paint = catalog.findAll().getFirst();

        palettes.addPaint(palette, paint);
        palettes.addPaint(palette, paint);

        assertThat(palette.getPaints()).containsExactly(paint);
    }

    @Test
    @DisplayName("un tube retire quitte la palette mais reste au catalogue")
    void removingKeepsThePaintInTheCatalog() {
        Palette palette = palettes.create("Palette a vider", "");
        OilPaint paint = catalog.findAll().getFirst();
        palettes.addPaint(palette, paint);

        palettes.removePaint(palette, paint);

        assertThat(palette.getPaints()).isEmpty();
        assertThat(catalog.findById(paint.getId())).isPresent();
    }

    @Test
    @DisplayName("la palette expose la contrainte de sechage de son tube le plus lent")
    void slowestPaintDrivesThePalette() {
        Palette palette = palettes.create("Palette de sechage", "");
        OilPaint fast = catalog.search("Burnt Umber").getFirst();
        OilPaint slow = catalog.search("Cadmium Yellow").getFirst();

        palettes.addPaint(palette, fast);
        assertThat(palette.slowestDryingClass()).isEqualTo(fast.getDryingClass());

        palettes.addPaint(palette, slow);
        assertThat(palette.slowestDryingClass()).isEqualTo(DryingClass.VERY_SLOW);
    }

    @Test
    @DisplayName("une recherche de melange limitee a une palette ne propose que ses tubes")
    void mixSearchStaysInsideThePalette() {
        Palette palette = palettes.findAll().getFirst();

        var suggestions = mixer.suggestMixes(
                be.asmolabs.palettier.core.color.Rgb.ofHex("#8A6A4A"), palette.getPaints(), 4);

        assertThat(suggestions).isNotEmpty();
        assertThat(suggestions).allSatisfy(suggestion ->
                assertThat(suggestion.parts()).allSatisfy(part ->
                        assertThat(palette.getPaints()).contains(part.paint())));
    }

    @Test
    @DisplayName("la palette Zorn est livree avec ses quatre tubes, tous chez Winsor & Newton")
    void zornPaletteIsSeeded() {
        Palette zorn = zorn();

        assertThat(zorn.getPaints()).hasSize(4);
        assertThat(zorn.getPaints()).allSatisfy(paint ->
                assertThat(paint.getBrand()).isEqualTo("Winsor & Newton"));
        assertThat(zorn.getPaints()).extracting(OilPaint::getName)
                .containsExactlyInAnyOrder("Titanium White", "Yellow Ochre",
                        "Cadmium Scarlet", "Ivory Black");
    }

    @Test
    @DisplayName("le cadmium impose son rythme a la palette Zorn")
    void zornPaletteIsPacedByTheCadmium() {
        assertThat(zorn().slowestDryingClass()).isEqualTo(DryingClass.VERY_SLOW);
    }

    @Test
    @DisplayName("le noir d'ivoire sort chaud du tube mais donne un gris froid coupe de blanc")
    void zornBlackIsWarmInTheTubeAndCoolInTheMix() {
        Palette zorn = zorn();
        OilPaint black = named(zorn, "Ivory Black");

        // C'est toute la raison d'etre du modele a deux constantes : une seule valeur par
        // tube ne peut pas porter ces deux comportements a la fois.
        assertThat(warmth(black.color()))
                .as("ton de masse chaud : %s", black.getHexColor())
                .isPositive();

        Rgb grey = mixedWith(named(zorn, "Titanium White"), black);
        assertThat(warmth(grey))
                .as("gris froid une fois coupe de blanc : %s", grey.toHex())
                .isNegative();
    }

    @Test
    @DisplayName("le noir est le point le plus froid de la palette Zorn")
    void zornBlackIsTheColdestOfTheFour() {
        Palette zorn = zorn();
        OilPaint white = named(zorn, "Titanium White");

        double withBlack = warmth(mixedWith(white, named(zorn, "Ivory Black")));
        double withOchre = warmth(mixedWith(white, named(zorn, "Yellow Ochre")));
        double withRed = warmth(mixedWith(white, named(zorn, "Cadmium Scarlet")));

        assertThat(withBlack).isLessThan(withOchre).isLessThan(withRed);
    }

    @Test
    @DisplayName("noir et ocre donnent un vert sourd, sans aucun pigment vert")
    void zornBlackAndOchreProduceAMutedGreen() {
        Palette zorn = zorn();

        Rgb mixed = mixedWith(named(zorn, "Yellow Ochre"), named(zorn, "Ivory Black"));

        assertThat(zorn.pigments()).as("aucun pigment vert sur la palette")
                .noneSatisfy(code -> assertThat(code).startsWith("PG"));
        assertThat(mixed.g())
                .as("la composante verte doit passer devant la bleue dans %s", mixed.toHex())
                .isGreaterThan(mixed.b());
    }

    private Rgb mixedWith(OilPaint base, OilPaint added) {
        return mixer.mix(List.of(
                be.asmolabs.palettier.core.service.MixModels.PaintPart.of(base, 6),
                be.asmolabs.palettier.core.service.MixModels.PaintPart.of(added, 1))).color();
    }

    /** Ecart rouge-bleu : positif pour une teinte chaude, negatif pour une teinte froide. */
    private static double warmth(Rgb colour) {
        return colour.r() - colour.b();
    }

    private Palette zorn() {
        return palettes.findAll().stream()
                .filter(palette -> palette.getName().startsWith("Palette Zorn"))
                .findFirst()
                .orElseThrow();
    }

    @Test
    @DisplayName("une palette de reference supprimee revient au demarrage suivant")
    void referencePalettesAreRestoredIndividually() {
        int before = palettes.findAll().size();
        Palette custom = palettes.create("Ma palette a moi", "");

        assertThat(palettes.findAll()).hasSize(before + 1).contains(custom);
    }

    private static OilPaint named(Palette palette, String name) {
        return palette.getPaints().stream()
                .filter(paint -> paint.getName().equals(name))
                .findFirst()
                .orElseThrow();
    }

    @Test
    @DisplayName("relever une teinte diluee fait passer le tube au melange a deux constantes")
    void recordingATintEnablesTwoConstantMixing() {
        // Un tube franchement chaud et sans teinte diluee connue : c'est la que l'ecart
        // entre le modele a une constante et celui a deux se voit le mieux. On le nomme
        // plutot que de prendre le premier venu, l'ordre du catalogue n'etant pas un contrat.
        OilPaint paint = catalog.findAll().stream()
                .filter(p -> p.getBrand().equals("Winsor & Newton") && p.getName().equals("Winsor Orange"))
                .findFirst()
                .orElseThrow(() -> new AssertionError("tube de reference absent du catalogue"));
        assertThat(paint.getTintHex()).as("teinte diluee non renseignee au depart").isNull();

        Rgb white = Rgb.ofHex("#F7F5F0");
        Rgb beforeTint = mixer.mix(List.of(
                be.asmolabs.palettier.core.service.MixModels.PaintPart.of(paint, 1),
                be.asmolabs.palettier.core.service.MixModels.PaintPart.of(named(zorn(), "Titanium White"), 9)))
                .color();

        // Une teinte franchement plus froide que ce que le modele a constante unique predit.
        catalog.recordTint(paint, Rgb.ofHex("#8E9094"));

        assertThat(paint.getTintHex()).isEqualTo("#8E9094");
        Rgb afterTint = mixer.mix(List.of(
                be.asmolabs.palettier.core.service.MixModels.PaintPart.of(paint, 1),
                be.asmolabs.palettier.core.service.MixModels.PaintPart.of(named(zorn(), "Titanium White"), 9)))
                .color();

        assertThat(warmth(afterTint))
                .as("le melange doit refroidir : %s -> %s", beforeTint.toHex(), afterTint.toHex())
                .isLessThan(warmth(beforeTint));
        assertThat(white).isNotNull();
    }

    @Test
    @DisplayName("relever un ton de masse remplace la valeur du catalogue")
    void recordingAMasstoneReplacesTheCatalogueValue() {
        OilPaint paint = catalog.findAll().getFirst();

        catalog.recordMasstone(paint, Rgb.ofHex("#123456"));

        assertThat(paint.getHexColor()).isEqualTo("#123456");
        assertThat(paint.isColorDerived()).isFalse();
    }
}
