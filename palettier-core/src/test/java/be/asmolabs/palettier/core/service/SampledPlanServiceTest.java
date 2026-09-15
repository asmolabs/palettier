package be.asmolabs.palettier.core.service;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatIllegalArgumentException;

import be.asmolabs.palettier.core.color.Rgb;
import be.asmolabs.palettier.core.domain.Palette;
import be.asmolabs.palettier.core.plan.PaintingPlan;
import be.asmolabs.palettier.core.service.SampledPlanService.ColourGroup;
import java.util.List;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.transaction.annotation.Transactional;

@SpringBootTest
@Transactional
class SampledPlanServiceTest {

    @Autowired
    private SampledPlanService sampled;

    @Autowired
    private PaletteService palettes;

    private Palette zorn() {
        return palettes.findAll().stream()
                .filter(p -> p.getName().startsWith("Palette Zorn"))
                .findFirst()
                .orElseThrow();
    }

    /** Cinq points d'une carnation, donnes volontairement en desordre. */
    private static ColourGroup skin() {
        return new ColourGroup("Peau", "Visage", List.of(
                Rgb.ofHex("#E8D2BE"),   // le plus clair
                Rgb.ofHex("#5A3B2E"),   // le plus sombre
                Rgb.ofHex("#C98F72"),   // median
                Rgb.ofHex("#8A5F4A"),
                Rgb.ofHex("#DCB69C")));
    }

    @Test
    @DisplayName("les roles suivent la clarte des points, pas l'ordre de prelevement")
    void rolesFollowLuminanceNotOrder() {
        PaintingPlan plan = sampled.plan("Buste", zorn(), List.of(skin()), 3);

        PaintingPlan.Zone zone = plan.zones().getFirst();
        assertThat(zone.layers()).extracting(PaintingPlan.Layer::role)
                .containsExactly("Ombre 2", "Ombre 1", "Base", "Lumiere 1", "Lumiere 2");
        assertThat(zone.base().target().toHex()).isEqualTo("#C98F72");
        assertThat(zone.shadows().getLast().target().toHex()).isEqualTo("#5A3B2E");
        assertThat(zone.highlights().getLast().target().toHex()).isEqualTo("#E8D2BE");
    }

    @Test
    @DisplayName("les couches vont bien du sombre au clair")
    void layersRunFromDarkToLight() {
        List<PaintingPlan.Layer> layers = sampled.plan("Buste", zorn(), List.of(skin()), 3)
                .zones().getFirst().layers();

        assertThat(layers).isSortedAccordingTo(
                (a, b) -> Double.compare(a.target().relativeLuminance(), b.target().relativeLuminance()));
    }

    @Test
    @DisplayName("chaque couche porte un melange calcule avec la palette")
    void everyLayerCarriesARecipe() {
        PaintingPlan plan = sampled.plan("Buste", zorn(), List.of(skin()), 3);

        assertThat(plan.zones().getFirst().layers())
                .allSatisfy(layer -> assertThat(layer.recipe()).isNotNull());
    }

    @Test
    @DisplayName("plusieurs groupes donnent plusieurs zones, les groupes vides sont ignores")
    void groupsBecomeZones() {
        PaintingPlan plan = sampled.plan("Buste", zorn(), List.of(
                skin(),
                new ColourGroup("Cape", "Tissu", List.of(Rgb.ofHex("#4A3A6A"), Rgb.ofHex("#8A7AB0"))),
                new ColourGroup("Vide", "", List.of())), 3);

        assertThat(plan.zones()).extracting(PaintingPlan.Zone::name).containsExactly("Peau", "Cape");
    }

    @Test
    @DisplayName("un seul point donne une base, et le dit")
    void oneSampleGivesABaseAndSaysSo() {
        PaintingPlan plan = sampled.plan("Buste", zorn(),
                List.of(new ColourGroup("Cuir", "", List.of(Rgb.ofHex("#6B4A2A")))), 3);

        PaintingPlan.Zone zone = plan.zones().getFirst();
        assertThat(zone.layers()).hasSize(1);
        assertThat(zone.base().role()).isEqualTo("Base");
        assertThat(zone.note()).contains("Relevez une ombre et une lumiere");
    }

    @Test
    @DisplayName("sans aucun releve, la demande est refusee avec un message clair")
    void nothingSampledIsRefused() {
        assertThatIllegalArgumentException()
                .isThrownBy(() -> sampled.plan("Buste", zorn(),
                        List.of(new ColourGroup("Peau", "", List.of())), 3))
                .withMessageContaining("Aucun point releve");
    }
}
