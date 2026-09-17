package be.asmolabs.palettier.ai;

import static org.assertj.core.api.Assertions.assertThat;

import be.asmolabs.palettier.ai.PlanDraft.AccentDraft;
import be.asmolabs.palettier.ai.PlanDraft.LayerDraft;
import be.asmolabs.palettier.ai.PlanDraft.ZoneDraft;
import be.asmolabs.palettier.core.color.Rgb;
import be.asmolabs.palettier.core.domain.DryingClass;
import be.asmolabs.palettier.core.domain.OilPaint;
import be.asmolabs.palettier.core.domain.Opacity;
import be.asmolabs.palettier.core.domain.Palette;
import be.asmolabs.palettier.core.plan.PaintingPlan;
import be.asmolabs.palettier.core.service.ColorMixService;
import java.util.List;
import java.util.Set;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.ObjectProvider;
import org.springframework.beans.factory.support.StaticListableBeanFactory;

/**
 * Ce que le service fait de la reponse du modele, sans moteur.
 *
 * <p>C'est la moitie du travail qui ne depend d'aucun serveur : combler une couche
 * oubliee, ecarter une variation vide, encaisser une couleur mal formee. Jusqu'ici rien
 * ne le verifiait hors de l'essai en direct, qui reclame Ollama et deux minutes.</p>
 */
class PlanEnrichmentTest {

    private final ObjectProvider<org.springframework.ai.chat.client.ChatClient.Builder> none =
            new StaticListableBeanFactory()
                    .getBeanProvider(org.springframework.ai.chat.client.ChatClient.Builder.class);

    private final PaintingPlanService service =
            new PaintingPlanService(none, null, new ColorMixService(), new AiSettings());

    private static Palette palette() {
        Palette palette = new Palette("Essai", "tests");
        palette.add(new OilPaint("W&N", "Titanium White", "644", "#F4F2EC",
                Opacity.OPAQUE, DryingClass.SLOW, 0.9, Set.of("PW6")));
        palette.add(new OilPaint("W&N", "Burnt Umber", "076", "#4A3427",
                Opacity.SEMI_OPAQUE, DryingClass.FAST, 0.8, Set.of("PBr7")));
        palette.add(new OilPaint("W&N", "Cadmium Red", "094", "#B22222",
                Opacity.OPAQUE, DryingClass.VERY_SLOW, 0.85, Set.of("PR108")));
        return palette;
    }

    private static LayerDraft hex(String value) {
        return new LayerDraft(value, "Glacis", "");
    }

    /** La zone unique du plan enrichi : tous ces essais n'en decrivent qu'une. */
    private PaintingPlan.Zone only(PlanDraft draft) {
        return service.enrich("Buste", palette(), draft, 3).zones().getFirst();
    }

    private static PlanDraft draftWith(LayerDraft base, LayerDraft shadow1, LayerDraft shadow2,
                                       LayerDraft highlight1, LayerDraft highlight2,
                                       AccentDraft... accents) {
        AccentDraft a1 = accents.length > 0 ? accents[0] : null;
        AccentDraft a2 = accents.length > 1 ? accents[1] : null;
        AccentDraft a3 = accents.length > 2 ? accents[2] : null;
        return new PlanDraft("Approche", List.of(new ZoneDraft("Visage", "Peau", "",
                base, shadow1, shadow2, highlight1, highlight2, a1, a2, a3)));
    }

    @Test
    @DisplayName("les cinq marches de l'echelle sont toujours presentes et nommees")
    void theFiveStepsAreAlwaysThere() {
        PaintingPlan.Zone zone = only(draftWith(hex("#C98F72"), hex("#8A5F4A"), hex("#5A3B2E"),
                hex("#E0B49A"), hex("#F2D8C4")));

        assertThat(zone.layers()).extracting(PaintingPlan.Layer::role)
                .containsExactly("Ombre 2", "Ombre 1", "Base", "Lumiere 1", "Lumiere 2");
    }

    @Test
    @DisplayName("une couche oubliee par le modele est interpolee, pas inventee")
    void amissingStepIsInterpolated() {
        // Ombre 1 absente : elle doit tomber entre la base et l'ombre profonde.
        PaintingPlan.Zone zone = only(draftWith(hex("#C98F72"), null, hex("#5A3B2E"),
                hex("#E0B49A"), hex("#F2D8C4")));

        PaintingPlan.Layer interpolated = zone.shadows().getFirst();
        assertThat(interpolated.role()).isEqualTo("Ombre 1");
        assertThat(interpolated.note()).contains("interpole");

        // Entre les deux, et non a cote : chaque canal est encadre.
        Rgb base = Rgb.ofHex("#C98F72");
        Rgb deep = Rgb.ofHex("#5A3B2E");
        Rgb got = interpolated.target();
        assertThat(got.r()).isBetween(Math.min(base.r(), deep.r()), Math.max(base.r(), deep.r()));
        assertThat(got.g()).isBetween(Math.min(base.g(), deep.g()), Math.max(base.g(), deep.g()));
        assertThat(got.b()).isBetween(Math.min(base.b(), deep.b()), Math.max(base.b(), deep.b()));
    }

    @Test
    @DisplayName("sans rien pour interpoler, la couche retombe sur la base et le dit")
    void withNothingToInterpolateItFallsBackToTheBase() {
        PaintingPlan.Zone zone = only(draftWith(hex("#C98F72"), null, null,
                hex("#E0B49A"), hex("#F2D8C4")));

        assertThat(zone.shadows()).allSatisfy(layer -> {
            assertThat(layer.target()).isEqualTo(Rgb.ofHex("#C98F72"));
            assertThat(layer.note()).contains("absente");
        });
    }

    @Test
    @DisplayName("une variation locale vide est ecartee, une variation sans nom en recoit un")
    void accentsAreFilteredAndNamed() {
        PaintingPlan.Zone zone = only(draftWith(hex("#C98F72"), hex("#8A5F4A"), hex("#5A3B2E"),
                hex("#E0B49A"), hex("#F2D8C4"),
                new AccentDraft("Rougeur des pommettes", "#C97A62", "Glacis", ""),
                new AccentDraft("Sans couleur", "", "Glacis", ""),
                new AccentDraft("", "#8FA3B0", "Filtre", "")));

        assertThat(zone.accents()).extracting(PaintingPlan.Layer::role)
                .containsExactly("Rougeur des pommettes", "Variation 2");
    }

    @Test
    @DisplayName("une couleur mal formee ne fait pas echouer le plan")
    void abadColourDoesNotSinkThePlan() {
        PaintingPlan.Zone zone = only(draftWith(hex("brun chaud"), hex("#8A5F4A"), hex("#5A3B2E"),
                hex("#E0B49A"), hex("#F2D8C4")));

        assertThat(zone.base().target()).isEqualTo(new Rgb(0.5, 0.5, 0.5));
        assertThat(zone.layers()).hasSize(5);
    }

    @Test
    @DisplayName("chaque couche porte un melange calcule et un ecart mesure")
    void everyLayerCarriesAMeasuredMix() {
        PaintingPlan.Zone zone = only(draftWith(hex("#C98F72"), hex("#8A5F4A"), hex("#5A3B2E"),
                hex("#E0B49A"), hex("#F2D8C4")));

        assertThat(zone.layers()).allSatisfy(layer -> {
            assertThat(layer.recipe()).as("melange calcule pour %s", layer.role()).isNotNull();
            assertThat(layer.deltaE()).isGreaterThanOrEqualTo(0);
        });
    }
}
