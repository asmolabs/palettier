package be.asmolabs.palettier.ui;

import static org.assertj.core.api.Assertions.assertThat;

import be.asmolabs.palettier.core.plan.PaintingPlan;
import be.asmolabs.palettier.core.color.Rgb;
import be.asmolabs.palettier.core.domain.DryingClass;
import be.asmolabs.palettier.core.domain.OilPaint;
import be.asmolabs.palettier.core.domain.Opacity;
import be.asmolabs.palettier.core.service.MixModels.MixSuggestion;
import be.asmolabs.palettier.core.service.MixModels.PaintPart;
import be.asmolabs.palettier.ui.export.PlanPdf;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import java.util.Set;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

class PlanPdfTest {

    private static final OilPaint OCHRE = new OilPaint("W&N", "Yellow Ochre", "744", "#C08A34",
            Opacity.SEMI_OPAQUE, DryingClass.MEDIUM, 0.6, Set.of("PY43"));
    private static final OilPaint WHITE = new OilPaint("W&N", "Titanium White", "644", "#F7F5F0",
            Opacity.OPAQUE, DryingClass.SLOW, 1.0, Set.of("PW6"));

    private static PaintingPlan.Layer layer(String role, String hex) {
        Rgb color = Rgb.ofHex(hex);
        MixSuggestion recipe = new MixSuggestion(
                List.of(PaintPart.of(OCHRE, 3), PaintPart.of(WHITE, 1)), color, 1.2);
        return new PaintingPlan.Layer(role, color, "Glacis", "Dans les creux.", recipe, color, 1.2);
    }

    private static PaintingPlan plan() {
        PaintingPlan.Zone zone = new PaintingPlan.Zone("Visage", "Peau",
                "Construire par glacis successifs.",
                layer("Base", "#C98F72"),
                List.of(layer("Ombre 1", "#8A5F4A"), layer("Ombre 2", "#5A3B2E")),
                List.of(layer("Lumiere 1", "#E0B49A"), layer("Lumiere 2", "#F2D8C4")));
        return new PaintingPlan("Buste de grognard", "Palette Zorn",
                "Palette courte, valeurs rompues.", List.of(zone, zone, zone));
    }

    @Test
    @DisplayName("le PDF est produit et contient les pages attendues")
    void writesAReadablePdf(@TempDir Path directory) throws Exception {
        Path file = directory.resolve("plan.pdf");

        new PlanPdf().write(plan(), file);

        assertThat(file).exists();
        assertThat(Files.size(file)).isGreaterThan(1000);
        // Signature PDF : le fichier doit s'ouvrir dans n'importe quel lecteur.
        assertThat(Files.readAllBytes(file)).startsWith("%PDF".getBytes());
    }

    @Test
    @DisplayName("les couches sont ordonnees de la plus sombre a la plus claire")
    void layersRunFromDarkToLight() {
        List<PaintingPlan.Layer> layers = plan().zones().getFirst().layers();

        assertThat(layers).hasSize(5);
        assertThat(layers).extracting(PaintingPlan.Layer::role)
                .containsExactly("Ombre 2", "Ombre 1", "Base", "Lumiere 1", "Lumiere 2");
    }

    @Test
    @DisplayName("un caractere hors du latin occidental ne fait pas echouer l'ecriture")
    void unsupportedCharactersDoNotBreakTheDocument(@TempDir Path directory) throws Exception {
        PaintingPlan withEmoji = new PaintingPlan("Buste — 中文 ✨", "Zorn",
                "Approche éclairée, naïve.", plan().zones());

        new PlanPdf().write(withEmoji, directory.resolve("exotique.pdf"));

        assertThat(directory.resolve("exotique.pdf")).exists();
    }

}
