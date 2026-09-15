package be.asmolabs.palettier.core.service;

import static org.assertj.core.api.Assertions.assertThat;

import be.asmolabs.palettier.core.domain.DryingClass;
import be.asmolabs.palettier.core.domain.LayerThickness;
import be.asmolabs.palettier.core.domain.Medium;
import be.asmolabs.palettier.core.domain.Ventilation;
import be.asmolabs.palettier.core.service.DryingModels.DryingContext;
import be.asmolabs.palettier.core.service.DryingModels.DryingEstimate;
import be.asmolabs.palettier.core.service.DryingModels.Workshop;
import java.time.Duration;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

class DryingTimeServiceTest {

    private final DryingTimeService service = new DryingTimeService(new DryingProperties());

    private static DryingContext context(Medium medium, double ratio, LayerThickness thickness, Workshop workshop) {
        return new DryingContext(DryingClass.MEDIUM, medium, ratio, thickness, workshop);
    }

    @Test
    @DisplayName("les jalons de sechage sont ordonnes du temps ouvert a la polymerisation")
    void milestonesAreOrdered() {
        DryingEstimate estimate = service.estimate(
                context(Medium.ODORLESS_THINNER, 0.8, LayerThickness.GLAZE, Workshop.standard()));

        assertThat(estimate.openTime()).isLessThan(estimate.touchDry());
        assertThat(estimate.touchDry()).isLessThan(estimate.recoat());
        assertThat(estimate.recoat()).isLessThan(estimate.throughDry());
        assertThat(estimate.throughDry()).isLessThan(estimate.fullCure());
    }

    @Test
    @DisplayName("le diluant accelere le sechage, l'huile de lin le ralentit")
    void mediumChangesDryingSpeed() {
        Duration pure = touchDry(Medium.NONE, 0.0);
        Duration thinned = touchDry(Medium.ODORLESS_THINNER, 0.8);
        Duration oiled = touchDry(Medium.LINSEED_OIL, 0.4);

        assertThat(thinned).isLessThan(pure);
        assertThat(oiled).isGreaterThan(pure);
    }

    @Test
    @DisplayName("une couche epaisse seche nettement plus lentement qu'un glacis")
    void thicknessDominatesDryingTime() {
        Duration glaze = touchDry(LayerThickness.GLAZE);
        Duration impasto = touchDry(LayerThickness.IMPASTO);

        assertThat(impasto).isGreaterThan(glaze.multipliedBy(5));
    }

    @Test
    @DisplayName("dix degres de moins doublent le temps de sechage")
    void temperatureHalvesOrDoublesDryingTime() {
        Duration warm = touchDry(new Workshop(20, 50, Ventilation.NORMAL));
        Duration cold = touchDry(new Workshop(10, 50, Ventilation.NORMAL));

        assertThat(cold.toMinutes()).isCloseTo(warm.toMinutes() * 2, org.assertj.core.data.Offset.offset(2L));
    }

    @Test
    @DisplayName("une dilution excessive declenche un avertissement")
    void excessiveThinningIsReported() {
        DryingEstimate estimate = service.estimate(
                context(Medium.ODORLESS_THINNER, 0.98, LayerThickness.GLAZE, Workshop.standard()));

        assertThat(estimate.advice()).anySatisfy(a -> assertThat(a).contains("limite utile"));
    }

    @Test
    @DisplayName("un atelier froid est signale au peintre")
    void coldWorkshopIsReported() {
        DryingEstimate estimate = service.estimate(
                context(Medium.NONE, 0.0, LayerThickness.NORMAL, new Workshop(12, 50, Ventilation.NORMAL)));

        assertThat(estimate.advice()).anySatisfy(a -> assertThat(a).contains("15 degres"));
    }

    private Duration touchDry(Medium medium, double ratio) {
        return service.estimate(context(medium, ratio, LayerThickness.NORMAL, Workshop.standard())).touchDry();
    }

    private Duration touchDry(LayerThickness thickness) {
        return service.estimate(context(Medium.NONE, 0.0, thickness, Workshop.standard())).touchDry();
    }

    private Duration touchDry(Workshop workshop) {
        return service.estimate(context(Medium.NONE, 0.0, LayerThickness.NORMAL, workshop)).touchDry();
    }
}
