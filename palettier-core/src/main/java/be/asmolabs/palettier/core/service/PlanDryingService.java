package be.asmolabs.palettier.core.service;

import be.asmolabs.palettier.core.domain.DryingClass;
import be.asmolabs.palettier.core.domain.OilPaint;
import be.asmolabs.palettier.core.domain.Technique;
import be.asmolabs.palettier.core.plan.PaintingPlan;
import be.asmolabs.palettier.core.service.DryingModels.DryingContext;
import be.asmolabs.palettier.core.service.DryingModels.DryingEstimate;
import be.asmolabs.palettier.core.service.DryingModels.Workshop;
import java.time.Duration;
import java.util.ArrayList;
import java.util.List;
import org.springframework.stereotype.Service;

/**
 * Deroule un plan de peinture dans le temps.
 *
 * <p>La difference avec le calcul d'une couche isolee tient a une chose : ici, on ne
 * demande pas au peintre quelle est la vitesse de sechage. Elle se deduit des pigments
 * reellement presents dans chaque melange. Un cadmium glisse dans un point lumineux
 * impose plusieurs jours a lui seul, et cela se voit sans avoir a y penser.</p>
 *
 * <p>Deux totaux sont produits, et leur ecart est l'information utile : le peintre qui
 * finit une zone avant d'attaquer la suivante attend la somme des zones, celui qui les
 * mene de front n'attend que la plus lente. Sur une figurine, la seconde facon est la
 * bonne, et c'est souvent plusieurs semaines de difference.</p>
 */
@Service
public class PlanDryingService {

    private final DryingTimeService dryingTimeService;

    public PlanDryingService(DryingTimeService dryingTimeService) {
        this.dryingTimeService = dryingTimeService;
    }

    /**
     * Une couche replacee dans le temps.
     *
     * @param startOffset delai entre le debut du travail sur la zone et cette couche
     * @param waitAfter   attente imposee avant la couche suivante de la meme zone
     */
    public record LayerSchedule(String role, String mix, Technique technique,
                                DryingClass dryingClass, DryingEstimate estimate,
                                Duration startOffset, Duration waitAfter) {
    }

    /** Une zone et son enchainement. */
    public record ZoneSchedule(String zone, String material, List<LayerSchedule> layers, Duration span) {

        /** Le pigment le plus lent rencontre dans la zone : c'est lui qui commande. */
        public DryingClass slowest() {
            return layers.stream().map(LayerSchedule::dryingClass)
                    .reduce(DryingClass.FAST, DryingClass::slowest);
        }
    }

    /**
     * @param sequential duree si l'on termine chaque zone avant d'attaquer la suivante
     * @param parallel   duree si l'on mene toutes les zones de front
     */
    public record Schedule(List<ZoneSchedule> zones, Duration sequential, Duration parallel,
                           List<String> advice) {
    }

    public Schedule schedule(PaintingPlan plan, Workshop workshop) {
        List<ZoneSchedule> zones = plan.zones().stream()
                .map(zone -> schedule(zone, workshop))
                .toList();

        Duration sequential = zones.stream()
                .map(ZoneSchedule::span)
                .reduce(Duration.ZERO, Duration::plus);
        Duration parallel = zones.stream()
                .map(ZoneSchedule::span)
                .max(Duration::compareTo)
                .orElse(Duration.ZERO);

        return new Schedule(zones, sequential, parallel, advice(zones, sequential, parallel));
    }

    private ZoneSchedule schedule(PaintingPlan.Zone zone, Workshop workshop) {
        // Les variations locales se posent apres l'echelle, et comptent dans le planning :
        // elles seront recouvertes comme le reste.
        List<PaintingPlan.Layer> layers = new ArrayList<>(zone.layers());
        layers.addAll(zone.accents());
        List<LayerSchedule> scheduled = new ArrayList<>();
        Duration offset = Duration.ZERO;

        for (int i = 0; i < layers.size(); i++) {
            PaintingPlan.Layer layer = layers.get(i);
            Technique technique = Technique.byLabel(layer.technique(), Technique.GLAZE);
            DryingClass drying = dryingClassOf(layer);

            DryingEstimate estimate = dryingTimeService.estimate(new DryingContext(
                    drying, technique.defaultMedium(), technique.defaultRatio(),
                    technique.typicalThickness(), workshop));

            boolean last = i == layers.size() - 1;
            // Une technique qui demande un support ferme attend le sechage a coeur ;
            // les autres se contentent du delai de recouvrement.
            Duration wait = last ? Duration.ZERO
                    : Technique.byLabel(layers.get(i + 1).technique(), Technique.GLAZE).requiresCuredBase()
                        ? estimate.throughDry() : estimate.recoat();

            scheduled.add(new LayerSchedule(layer.role(),
                    layer.recipe() == null ? "" : layer.recipe().describe(),
                    technique, drying, estimate, offset, wait));
            offset = offset.plus(wait);
        }
        return new ZoneSchedule(zone.name(), zone.material(), List.copyOf(scheduled), offset);
    }

    /**
     * La vitesse de sechage d'une couche, lue dans les tubes qui la composent.
     *
     * <p>C'est tout l'interet de partir d'un projet plutot que d'un reglage : le peintre
     * n'a pas a savoir que son blanc de titane est lent, l'application le sait.</p>
     */
    public static DryingClass dryingClassOf(PaintingPlan.Layer layer) {
        return layer.recipe() == null ? DryingClass.MEDIUM : layer.recipe().dryingClass();
    }

    private static List<String> advice(List<ZoneSchedule> zones, Duration sequential, Duration parallel) {
        List<String> advice = new ArrayList<>();
        if (zones.isEmpty()) {
            return List.of();
        }

        if (sequential.compareTo(parallel) > 0) {
            advice.add(("Menez les zones de front plutot que l'une apres l'autre : pendant qu'une "
                    + "couche seche, les autres zones avancent. Il n'y a rien a gagner a attendre."));
        }

        zones.stream()
                .max((a, b) -> a.span().compareTo(b.span()))
                .filter(slowest -> zones.size() > 1)
                .ifPresent(slowest -> advice.add(
                        "La zone %s commande le planning a elle seule : commencez par elle."
                                .formatted(slowest.zone())));

        zones.stream()
                .filter(zone -> zone.slowest().compareTo(DryingClass.SLOW) >= 0)
                .forEach(zone -> advice.add(
                        "%s contient un pigment a sechage %s : c'est lui qui impose les delais de cette zone."
                                .formatted(zone.zone(), zone.slowest().label().toLowerCase())));

        return List.copyOf(advice);
    }
}
