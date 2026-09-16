package be.asmolabs.palettier.core.service;

import be.asmolabs.palettier.core.domain.DryingClass;
import be.asmolabs.palettier.core.domain.Project;
import be.asmolabs.palettier.core.domain.ProjectLayer;
import be.asmolabs.palettier.core.domain.ProjectZone;
import be.asmolabs.palettier.core.domain.Technique;
import be.asmolabs.palettier.core.repository.ProjectRepository;
import be.asmolabs.palettier.core.service.DryingModels.DryingContext;
import be.asmolabs.palettier.core.service.DryingModels.DryingEstimate;
import be.asmolabs.palettier.core.service.DryingModels.Workshop;
import java.time.Duration;
import java.time.Instant;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.Optional;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

/**
 * L'etabli : ou en sont les pieces en cours, et laquelle peut etre reprise maintenant.
 *
 * <p>C'est la question que pose le peintre en entrant dans son atelier, et la seule que
 * les autres ecrans ne savaient pas traiter. Le sechage y etait calcule dans l'abstrait
 * -- "cette couche demandera trois jours" -- faute de savoir quand elle avait ete posee.
 * Une fois la pose consignee sur la couche, la meme formule repond enfin au present :
 * recouvrable depuis hier soir, ou dans six heures.</p>
 *
 * <p>Les conditions employees sont celles figees au moment de la pose, et non celles de
 * l'atelier aujourd'hui : l'huile a seche dans l'atelier qu'elle a connu. Chauffer la
 * piece a posteriori ne rattrape pas une semaine passee au froid.</p>
 *
 * <p>Une piece est prete des qu'<em>une</em> de ses zones l'est, et non quand toutes le
 * sont. C'est la meme raison qui fait conseiller ailleurs de mener les zones de front :
 * pendant qu'une joue seche, la cape avance.</p>
 */
@Service
@Transactional(readOnly = true)
public class WorkbenchService {

    /**
     * Ou en est une couche posee.
     *
     * <p>Les etats se suivent dans l'ordre du temps : une couche les traverse tous, sans
     * jamais revenir en arriere. L'interet n'est pas de nommer un etat mais de dire ce
     * qu'il autorise -- c'est la question du peintre.</p>
     */
    public enum Stage {

        OPEN("Encore ouverte",
                "Fondus et reprises dans le frais sont encore possibles."),
        SETTING("En prise",
                "Elle ne se travaille plus sans l'arracher : ne la touchez pas."),
        TOUCH_DRY("Seche au toucher",
                "Elle ne marque plus, mais une couche posee dessus la releverait encore."),
        RECOATABLE("Recouvrable",
                "La couche suivante peut etre posee sans relever celle-ci."),
        THROUGH_DRY("Seche a coeur",
                "Manipulation, masquage et montage possibles."),
        CURED("Polymerisee",
                "Vernis final possible.");

        private final String label;
        private final String meaning;

        Stage(String label, String meaning) {
            this.label = label;
            this.meaning = meaning;
        }

        public String label() {
            return label;
        }

        /** Ce que cet etat autorise, en clair. */
        public String meaning() {
            return meaning;
        }

        @Override
        public String toString() {
            return label;
        }
    }

    /**
     * Une couche posee, replacee dans le present.
     *
     * @param untilNext attente restante avant de pouvoir poser la couche suivante, nulle
     *                  si l'on peut y aller -- ou s'il n'y a plus rien a poser
     * @param nextRole  role de la couche qui attend, ou {@code null} si la zone est finie
     */
    public record CoatState(String zone, String role, String targetHex, Instant appliedAt,
                            DryingClass dryingClass, DryingEstimate estimate, Stage stage,
                            String nextRole, Duration untilNext) {

        public boolean readyForNext() {
            return untilNext.isZero();
        }
    }

    /**
     * Une zone et son avancement.
     *
     * @param last derniere couche posee, ou {@code null} si la zone n'est pas commencee
     */
    public record ZoneState(String name, String material, int applied, int total, CoatState last) {

        /** Toutes les couches prevues sont posees : il n'y a plus rien a y faire. */
        public boolean done() {
            return total > 0 && applied == total;
        }

        public boolean notStarted() {
            return applied == 0;
        }

        /** Du travail est possible tout de suite sur cette zone. */
        public boolean readyNow() {
            return !done() && (last == null || last.readyForNext());
        }

        /** Attente avant que cette zone redevienne disponible, nulle si elle l'est deja. */
        public Duration remaining() {
            return readyNow() || done() ? Duration.ZERO : last.untilNext();
        }
    }

    /** Une piece en cours et l'etat de chacune de ses zones. */
    public record PieceState(Project project, List<ZoneState> zones) {

        public List<ZoneState> readyZones() {
            return zones.stream().filter(ZoneState::readyNow).toList();
        }

        public boolean readyNow() {
            return !readyZones().isEmpty();
        }

        /** La piece est peinte : toutes ses zones le sont. */
        public boolean done() {
            return !zones.isEmpty() && zones.stream().allMatch(ZoneState::done);
        }

        /**
         * Delai avant que la piece redevienne disponible, vide si elle l'est deja ou si
         * elle est finie. C'est la plus courte des attentes : il suffit qu'une zone se
         * libere pour qu'on puisse reprendre la piece.
         */
        public Optional<Duration> nextAvailability() {
            if (readyNow() || done()) {
                return Optional.empty();
            }
            return zones.stream()
                    .filter(zone -> !zone.done())
                    .map(ZoneState::remaining)
                    .min(Duration::compareTo);
        }

        public int appliedCoats() {
            return zones.stream().mapToInt(ZoneState::applied).sum();
        }

        public int totalCoats() {
            return zones.stream().mapToInt(ZoneState::total).sum();
        }
    }

    /** L'etabli entier, les pieces reprenables en tete. */
    public record Bench(List<PieceState> pieces) {

        public List<PieceState> ready() {
            return pieces.stream().filter(PieceState::readyNow).toList();
        }

        public List<PieceState> waiting() {
            return pieces.stream().filter(piece -> !piece.readyNow() && !piece.done()).toList();
        }

        public List<PieceState> done() {
            return pieces.stream().filter(PieceState::done).toList();
        }

        /** Delai avant que quoi que ce soit se libere, vide si du travail attend deja. */
        public Optional<Duration> nextAvailability() {
            return waiting().stream()
                    .map(PieceState::nextAvailability)
                    .flatMap(Optional::stream)
                    .min(Duration::compareTo);
        }
    }

    private final ProjectRepository projects;
    private final DryingTimeService dryingTimeService;

    public WorkbenchService(ProjectRepository projects, DryingTimeService dryingTimeService) {
        this.projects = projects;
        this.dryingTimeService = dryingTimeService;
    }

    public Bench bench() {
        return bench(Instant.now());
    }

    /** @param now instant de reference, parametre pour que l'etat soit verifiable */
    public Bench bench(Instant now) {
        List<PieceState> pieces = projects.findAllByOrderByCreatedAtDesc().stream()
                .map(project -> state(project, now))
                .sorted(byUrgency())
                .toList();
        return new Bench(pieces);
    }

    /**
     * Les pieces reprenables d'abord, puis celles dont l'attente est la plus courte, et
     * les pieces finies en dernier. Au sein d'un groupe, l'ordre d'origine est conserve :
     * la plus recemment commencee reste en tete.
     */
    private static Comparator<PieceState> byUrgency() {
        return Comparator.<PieceState>comparingInt(piece -> {
                    if (piece.readyNow()) {
                        return 0;
                    }
                    return piece.done() ? 2 : 1;
                })
                .thenComparing(piece -> piece.nextAvailability().orElse(Duration.ZERO));
    }

    private PieceState state(Project project, Instant now) {
        List<ZoneState> zones = project.getZones().stream()
                .map(zone -> state(zone, now))
                .toList();
        return new PieceState(project, zones);
    }

    private ZoneState state(ProjectZone zone, Instant now) {
        List<ProjectLayer> layers = zone.getLayers();
        List<ProjectLayer> applied = layers.stream().filter(ProjectLayer::isApplied).toList();

        // La derniere posee au sens du temps, et non du rang : le peintre ne suit pas
        // forcement l'ordre du plan, et c'est la couche la plus fraiche qui commande.
        ProjectLayer last = applied.stream()
                .max(Comparator.comparing(ProjectLayer::getAppliedAt))
                .orElse(null);
        ProjectLayer next = layers.stream().filter(layer -> !layer.isApplied()).findFirst().orElse(null);

        CoatState state = last == null ? null : coat(zone, last, next, now);
        return new ZoneState(zone.getName(), zone.getMaterial(), applied.size(), layers.size(), state);
    }

    private CoatState coat(ProjectZone zone, ProjectLayer layer, ProjectLayer next, Instant now) {
        DryingEstimate estimate = dryingTimeService.estimate(context(layer));
        Duration elapsed = Duration.between(layer.getAppliedAt(), now);
        if (elapsed.isNegative()) {
            elapsed = Duration.ZERO;
        }

        // Une technique qui demande un support ferme attend le sechage a coeur ; les
        // autres se contentent du delai de recouvrement. Meme regle que le planning
        // previsionnel, appliquee ici a la couche qui attend reellement son tour.
        Duration gate = next == null ? Duration.ZERO
                : Technique.byLabel(next.getTechnique(), Technique.GLAZE).requiresCuredBase()
                    ? estimate.throughDry() : estimate.recoat();
        Duration remaining = gate.minus(elapsed);

        return new CoatState(zone.getName(), layer.getRole(), layer.getTargetHex(),
                layer.getAppliedAt(), dryingClass(layer), estimate, stageOf(elapsed, estimate),
                next == null ? null : next.getRole(),
                remaining.isNegative() ? Duration.ZERO : remaining);
    }

    /**
     * Les conditions de la pose. Le repli sur l'atelier de reference ne sert qu'aux
     * couches marquees avant que ces colonnes existent : une pose consignee les porte
     * toujours.
     */
    private static DryingContext context(ProjectLayer layer) {
        Workshop workshop = layer.getAppliedTemperature() == null
                || layer.getAppliedHumidity() == null
                || layer.getAppliedVentilation() == null
                ? Workshop.standard()
                : new Workshop(layer.getAppliedTemperature(), layer.getAppliedHumidity(),
                        layer.getAppliedVentilation());

        Technique technique = Technique.byLabel(layer.getTechnique(), Technique.GLAZE);
        return new DryingContext(dryingClass(layer), technique.defaultMedium(),
                technique.defaultRatio(), technique.typicalThickness(), workshop);
    }

    private static DryingClass dryingClass(ProjectLayer layer) {
        return layer.getAppliedDryingClass() == null ? DryingClass.MEDIUM : layer.getAppliedDryingClass();
    }

    private static Stage stageOf(Duration elapsed, DryingEstimate estimate) {
        List<Duration> milestones = new ArrayList<>(List.of(estimate.openTime(), estimate.touchDry(),
                estimate.recoat(), estimate.throughDry(), estimate.fullCure()));
        Stage[] stages = Stage.values();
        for (int i = 0; i < milestones.size(); i++) {
            if (elapsed.compareTo(milestones.get(i)) < 0) {
                return stages[i];
            }
        }
        return Stage.CURED;
    }
}
