package be.asmolabs.palettier.core.service;

import static org.assertj.core.api.Assertions.assertThat;

import be.asmolabs.palettier.core.color.Rgb;
import be.asmolabs.palettier.core.domain.DryingClass;
import be.asmolabs.palettier.core.domain.Project;
import be.asmolabs.palettier.core.domain.Ventilation;
import be.asmolabs.palettier.core.plan.PaintingPlan;
import be.asmolabs.palettier.core.service.DryingModels.Workshop;
import be.asmolabs.palettier.core.service.WorkbenchService.Bench;
import be.asmolabs.palettier.core.service.WorkbenchService.PieceState;
import be.asmolabs.palettier.core.service.WorkbenchService.Stage;
import be.asmolabs.palettier.core.service.WorkbenchService.ZoneState;
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
class WorkbenchServiceTest {

    private static final Instant NOW = Instant.parse("2026-03-01T10:00:00Z");

    @Autowired
    private ProjectService projects;

    @Autowired
    private WorkbenchService workbench;

    private static PaintingPlan.Layer layer(String role, String hex) {
        return new PaintingPlan.Layer(role, Rgb.ofHex(hex), "Glacis", "", null, Rgb.ofHex(hex), 0);
    }

    private static PaintingPlan.Zone zone(String name) {
        return new PaintingPlan.Zone(name, "Peau", "",
                layer("Base", "#C98F72"),
                List.of(layer("Ombre 1", "#8A5F4A")),
                List.of(layer("Lumiere 1", "#E0B49A")));
    }

    /** Deux zones de trois couches : de quoi verifier qu'elles avancent independamment. */
    private Project piece(String name) {
        PaintingPlan plan = new PaintingPlan("Buste", "", "", List.of(zone("Visage"), zone("Cape")));
        return projects.save(plan, null, name);
    }

    private PieceState state(Project project, Instant now) {
        return workbench.bench(now).pieces().stream()
                .filter(piece -> piece.project().getId().equals(project.getId()))
                .findFirst()
                .orElseThrow();
    }

    @Test
    @DisplayName("une piece dont rien n'est peint est disponible tout de suite")
    void nothingPaintedMeansNothingToWaitFor() {
        PieceState piece = state(piece("Rien de pose"), NOW);

        assertThat(piece.zones()).allMatch(ZoneState::notStarted);
        assertThat(piece.readyNow()).isTrue();
        assertThat(piece.nextAvailability()).isEmpty();
        assertThat(piece.appliedCoats()).isZero();
        assertThat(piece.totalCoats()).isEqualTo(6);
    }

    @Test
    @DisplayName("une couche fraiche met sa zone en attente, sans bloquer les autres")
    void afreshCoatHoldsItsOwnZoneOnly() {
        Project project = piece("Visage frais");
        project = projects.markApplied(project, 0, 0, Workshop.standard(), DryingClass.SLOW, NOW);

        PieceState piece = state(project, NOW.plus(Duration.ofMinutes(30)));
        ZoneState painted = piece.zones().getFirst();
        ZoneState untouched = piece.zones().get(1);

        assertThat(painted.readyNow()).isFalse();
        assertThat(painted.last().stage()).isEqualTo(Stage.OPEN);
        assertThat(painted.remaining()).isPositive();
        assertThat(untouched.readyNow()).isTrue();

        // Il suffit qu'une zone soit libre pour que la piece le soit : pendant que le
        // visage seche, la cape avance.
        assertThat(piece.readyNow()).isTrue();
    }

    @Test
    @DisplayName("passe le delai, la zone redevient disponible et l'annonce")
    void timePassingFreesTheZone() {
        Project project = piece("Visage sec");
        project = projects.markApplied(project, 0, 0, Workshop.standard(), DryingClass.FAST, NOW);
        project = projects.markApplied(project, 1, 0, Workshop.standard(), DryingClass.FAST, NOW);

        PieceState wet = state(project, NOW.plus(Duration.ofHours(2)));
        assertThat(wet.readyNow()).isFalse();
        assertThat(wet.nextAvailability()).isPresent();

        PieceState dry = state(project, NOW.plus(Duration.ofDays(30)));
        assertThat(dry.zones()).allMatch(ZoneState::readyNow);
        assertThat(dry.zones().getFirst().last().stage()).isEqualTo(Stage.CURED);
        assertThat(dry.nextAvailability()).isEmpty();
    }

    @Test
    @DisplayName("le sechage court avec les conditions de la pose, pas avec celles d'aujourd'hui")
    void conditionsAreFrozenWithTheCoat() {
        Project cold = projects.markApplied(piece("Atelier froid"), 0, 0,
                new Workshop(10, 80, Ventilation.CONFINED), DryingClass.MEDIUM, NOW);
        Project warm = projects.markApplied(piece("Atelier chaud"), 0, 0,
                new Workshop(28, 35, Ventilation.GOOD), DryingClass.MEDIUM, NOW);

        Instant later = NOW.plus(Duration.ofDays(2));
        Duration coldWait = state(cold, later).zones().getFirst().remaining();
        Duration warmWait = state(warm, later).zones().getFirst().remaining();

        // Meme couche, meme age : seul l'atelier du jour de la pose les separe.
        assertThat(coldWait).isGreaterThan(warmWait);
    }

    @Test
    @DisplayName("une zone entierement peinte ne reclame plus rien")
    void afinishedZoneAsksForNothing() {
        Project project = piece("Visage fini");
        for (int layerIndex = 0; layerIndex < 3; layerIndex++) {
            project = projects.markApplied(project, 0, layerIndex,
                    Workshop.standard(), DryingClass.FAST, NOW);
        }

        ZoneState finished = state(project, NOW.plus(Duration.ofMinutes(10))).zones().getFirst();
        assertThat(finished.done()).isTrue();
        assertThat(finished.readyNow()).isFalse();
        assertThat(finished.remaining()).isZero();
        assertThat(finished.last().nextRole()).isNull();
    }

    @Test
    @DisplayName("l'etabli met en tete ce qui peut etre repris")
    void whatCanBeResumedComesFirst() {
        Project waiting = piece("Tout frais");
        for (int zoneIndex = 0; zoneIndex < 2; zoneIndex++) {
            waiting = projects.markApplied(waiting, zoneIndex, 0,
                    Workshop.standard(), DryingClass.VERY_SLOW, NOW);
        }
        piece("Pas commence");

        Bench bench = workbench.bench(NOW.plus(Duration.ofHours(1)));

        assertThat(bench.ready()).extracting(piece -> piece.project().getName())
                .containsExactly("Pas commence");
        assertThat(bench.waiting()).extracting(piece -> piece.project().getName())
                .containsExactly("Tout frais");
        assertThat(bench.pieces().getFirst().project().getName()).isEqualTo("Pas commence");
        assertThat(bench.nextAvailability()).isPresent();
    }

    @Test
    @DisplayName("annuler une pose remet la couche a peindre")
    void aMistakeCanBeUndone() {
        Project project = piece("Fausse manoeuvre");
        project = projects.markApplied(project, 0, 0, Workshop.standard(), DryingClass.SLOW, NOW);
        assertThat(state(project, NOW).zones().getFirst().notStarted()).isFalse();

        project = projects.clearApplied(project, 0, 0);
        assertThat(state(project, NOW).zones().getFirst().notStarted()).isTrue();
    }
}
