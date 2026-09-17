package be.asmolabs.palettier.core.service;

import static org.assertj.core.api.Assertions.assertThat;

import be.asmolabs.palettier.core.color.Rgb;
import be.asmolabs.palettier.core.domain.DryingClass;
import be.asmolabs.palettier.core.domain.Project;
import be.asmolabs.palettier.core.plan.PaintingPlan;
import be.asmolabs.palettier.core.service.DryingModels.Workshop;
import be.asmolabs.palettier.core.service.ReadinessWatch.Freed;
import java.time.Duration;
import java.time.Instant;
import java.util.List;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.transaction.annotation.Transactional;

@SpringBootTest
@Transactional
class ReadinessWatchTest {

    private static final Instant NOW = Instant.parse("2026-03-01T10:00:00Z");

    @Autowired
    private ProjectService projects;

    @Autowired
    private WorkbenchService workbench;

    private final ReadinessWatch watch = new ReadinessWatch();

    private static PaintingPlan.Layer layer(String role, String hex) {
        return new PaintingPlan.Layer(role, Rgb.ofHex(hex), "Glacis", "", null, Rgb.ofHex(hex), 0);
    }

    private Project piece(String name) {
        PaintingPlan.Zone zone = new PaintingPlan.Zone("Visage", "Peau", "",
                layer("Base", "#C98F72"),
                List.of(layer("Ombre 1", "#8A5F4A")),
                List.of(layer("Lumiere 1", "#E0B49A")));
        return projects.save(new PaintingPlan("Buste", "", "", List.of(zone)), null, name);
    }

    @Test
    @DisplayName("le premier regard n'annonce rien : tout est deja a l'ecran")
    void thefirstLookAnnouncesNothing() {
        piece("Rien de pose");

        assertThat(watch.newlyReady(workbench.bench(NOW))).isEmpty();
    }

    @Test
    @DisplayName("une zone qui se libere est annoncee une fois, puis se tait")
    void azoneIsAnnouncedOnce() {
        Project project = piece("Visage frais");
        projects.markApplied(project, 0, 0, Workshop.standard(), DryingClass.FAST, NOW);

        // La couche est fraiche : rien de disponible, et c'est le passage qui amorce.
        Instant wet = NOW.plus(Duration.ofMinutes(20));
        assertThat(watch.newlyReady(workbench.bench(wet))).isEmpty();
        assertThat(watch.newlyReady(workbench.bench(wet))).isEmpty();

        // Le temps passe : la zone bascule, et on l'apprend.
        Instant dry = NOW.plus(Duration.ofDays(30));
        assertThat(watch.newlyReady(workbench.bench(dry)))
                .extracting(Freed::label)
                .containsExactly("Visage frais - Visage");

        // Elle reste disponible, mais on ne le repete pas : une alerte qui se repete
        // cesse d'etre lue.
        assertThat(watch.newlyReady(workbench.bench(dry))).isEmpty();
    }

    @Test
    @DisplayName("oublier ce qui a ete vu remet la surveillance a zero")
    void resettingForgetsEverything() {
        Project project = piece("A oublier");
        projects.markApplied(project, 0, 0, Workshop.standard(), DryingClass.FAST, NOW);
        Instant dry = NOW.plus(Duration.ofDays(30));

        watch.newlyReady(workbench.bench(NOW.plus(Duration.ofMinutes(20))));
        assertThat(watch.newlyReady(workbench.bench(dry))).isNotEmpty();

        watch.reset();
        assertThat(watch.newlyReady(workbench.bench(dry))).as("le regard qui reamorce").isEmpty();
        assertThat(watch.newlyReady(workbench.bench(dry))).isEmpty();
    }
}
