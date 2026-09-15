package be.asmolabs.palettier.core.service;

import static org.assertj.core.api.Assertions.assertThat;

import be.asmolabs.palettier.core.color.Rgb;
import be.asmolabs.palettier.core.domain.DryingClass;
import be.asmolabs.palettier.core.domain.OilPaint;
import be.asmolabs.palettier.core.domain.Opacity;
import be.asmolabs.palettier.core.domain.Technique;
import be.asmolabs.palettier.core.plan.PaintingPlan;
import be.asmolabs.palettier.core.service.DryingModels.Workshop;
import be.asmolabs.palettier.core.service.MixModels.MixSuggestion;
import be.asmolabs.palettier.core.service.MixModels.PaintPart;
import be.asmolabs.palettier.core.service.PlanDryingService.Schedule;
import java.util.List;
import java.util.Set;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

class PlanDryingServiceTest {

    private final PlanDryingService service =
            new PlanDryingService(new DryingTimeService(new DryingProperties()));

    private static final OilPaint UMBER = new OilPaint("W&N", "Burnt Umber", "076", "#4A3427",
            Opacity.SEMI_OPAQUE, DryingClass.FAST, 0.8, Set.of("PBr7"));
    private static final OilPaint CADMIUM = new OilPaint("W&N", "Cadmium Yellow", "108", "#F5B800",
            Opacity.OPAQUE, DryingClass.VERY_SLOW, 0.8, Set.of("PY35"));

    private static PaintingPlan.Layer layer(String role, Technique technique, OilPaint paint) {
        Rgb colour = paint.color();
        MixSuggestion recipe = new MixSuggestion(List.of(PaintPart.of(paint, 1)), colour, 0.5);
        return new PaintingPlan.Layer(role, colour, technique.label(), "", recipe, colour, 0.5);
    }

    private static PaintingPlan.Zone zone(String name, OilPaint highlightPaint) {
        return new PaintingPlan.Zone(name, "", "",
                layer("Base", Technique.BASE_LAYER, UMBER),
                List.of(layer("Ombre 1", Technique.GLAZE, UMBER)),
                List.of(layer("Lumiere 1", Technique.HIGHLIGHT, highlightPaint)));
    }

    @Test
    @DisplayName("la vitesse de sechage vient des pigments du melange, on ne la demande pas")
    void dryingClassComesFromThePaints() {
        Schedule schedule = service.schedule(
                new PaintingPlan("Buste", "Zorn", "", List.of(zone("Peau", CADMIUM))),
                Workshop.standard());

        var layers = schedule.zones().getFirst().layers();
        assertThat(layers).extracting(PlanDryingService.LayerSchedule::dryingClass)
                .containsExactly(DryingClass.FAST, DryingClass.FAST, DryingClass.VERY_SLOW);
    }

    @Test
    @DisplayName("un cadmium dans une lumiere allonge le planning de la zone")
    void aSlowPigmentStretchesTheZone() {
        var fast = service.schedule(
                new PaintingPlan("B", "Z", "", List.of(zone("Peau", UMBER))), Workshop.standard());
        var slow = service.schedule(
                new PaintingPlan("B", "Z", "", List.of(zone("Peau", CADMIUM))), Workshop.standard());

        // La derniere couche n'ajoute pas d'attente ; c'est son sechage a coeur qui compte.
        assertThat(slow.zones().getFirst().slowest()).isEqualTo(DryingClass.VERY_SLOW);
        assertThat(fast.zones().getFirst().slowest()).isEqualTo(DryingClass.FAST);
    }

    @Test
    @DisplayName("mener les zones de front est plus court que les enchainer")
    void workingZonesInParallelIsShorter() {
        Schedule schedule = service.schedule(new PaintingPlan("Buste", "Zorn", "",
                List.of(zone("Peau", UMBER), zone("Cape", UMBER), zone("Cuir", UMBER))),
                Workshop.standard());

        assertThat(schedule.parallel()).isLessThan(schedule.sequential());
        assertThat(schedule.sequential()).isEqualTo(
                schedule.zones().stream().map(PlanDryingService.ZoneSchedule::span)
                        .reduce(java.time.Duration.ZERO, java.time.Duration::plus));
        assertThat(schedule.advice()).anySatisfy(a -> assertThat(a).contains("de front"));
    }

    @Test
    @DisplayName("un atelier froid allonge tout le planning")
    void aColdWorkshopStretchesEverything() {
        PaintingPlan plan = new PaintingPlan("B", "Z", "", List.of(zone("Peau", UMBER)));

        var warm = service.schedule(plan, Workshop.standard());
        var cold = service.schedule(plan,
                new Workshop(10, 50, be.asmolabs.palettier.core.domain.Ventilation.NORMAL));

        assertThat(cold.parallel()).isGreaterThan(warm.parallel());
    }

    @Test
    @DisplayName("une couche sans recette garde un sechage moyen plutot que d'echouer")
    void aLayerWithoutARecipeFallsBack() {
        PaintingPlan.Layer orphan = new PaintingPlan.Layer("Base", Rgb.ofHex("#808080"),
                Technique.GLAZE.label(), "", null, Rgb.ofHex("#808080"), 0);
        PaintingPlan plan = new PaintingPlan("B", "Z", "",
                List.of(new PaintingPlan.Zone("Peau", "", "", orphan, List.of(), List.of())));

        Schedule schedule = service.schedule(plan, Workshop.standard());

        assertThat(schedule.zones().getFirst().layers().getFirst().dryingClass())
                .isEqualTo(DryingClass.MEDIUM);
    }
}
