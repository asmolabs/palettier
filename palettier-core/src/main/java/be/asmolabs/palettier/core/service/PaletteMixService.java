package be.asmolabs.palettier.core.service;

import be.asmolabs.palettier.core.color.Rgb;
import be.asmolabs.palettier.core.domain.DryingClass;
import be.asmolabs.palettier.core.domain.LayerThickness;
import be.asmolabs.palettier.core.domain.Medium;
import be.asmolabs.palettier.core.domain.PaletteMix;
import be.asmolabs.palettier.core.repository.PaletteMixRepository;
import be.asmolabs.palettier.core.service.DryingModels.DryingContext;
import be.asmolabs.palettier.core.service.DryingModels.DryingEstimate;
import be.asmolabs.palettier.core.service.DryingModels.Workshop;
import java.time.Duration;
import java.time.Instant;
import java.util.List;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

/**
 * Ce qui est pose sur la palette, et pour combien de temps encore.
 *
 * <p>Une pate sur la palette n'est pas une couche sur la piece : elle n'est pas etalee,
 * elle forme un tas. Elle prend donc bien plus lentement, et c'est pour cela que le temps
 * ouvert se calcule ici avec une epaisseur chargee. C'est le meme modele que partout
 * ailleurs, applique a une autre forme.</p>
 *
 * <p>Le peintre n'a rien a saisir de plus qu'un nom : l'heure et l'atelier sont pris au
 * moment ou il declare le melange.</p>
 */
@Service
@Transactional(readOnly = true)
public class PaletteMixService {

    /** Un tas sur la palette, pas un film sur la piece. */
    private static final LayerThickness ON_THE_PALETTE = LayerThickness.THICK;

    private final PaletteMixRepository repository;
    private final DryingTimeService dryingTimeService;

    public PaletteMixService(PaletteMixRepository repository, DryingTimeService dryingTimeService) {
        this.repository = repository;
        this.dryingTimeService = dryingTimeService;
    }

    /**
     * Un melange et son etat.
     *
     * @param workable   temps restant avant qu'il ne se travaille plus, nul s'il a pris
     * @param unusable   temps restant avant qu'il ne serve plus a rien du tout
     */
    public record MixState(PaletteMix mix, Duration workable, Duration unusable) {

        /** Vrai tant qu'on peut encore s'en servir pour fondre et etaler. */
        public boolean isOpen() {
            return !workable.isZero();
        }

        /** Vrai quand il a pris et ne merite plus de place sur la palette. */
        public boolean isSpent() {
            return unusable.isZero();
        }
    }

    public List<PaletteMix> findAll() {
        return repository.findAllByOrderByMixedAtDesc();
    }

    /** Declare un melange pose maintenant, dans l'atelier tel qu'il est. */
    @Transactional
    public PaletteMix record(String name, Rgb color, String recipe, DryingClass dryingClass,
                             Workshop workshop) {
        return record(name, color, recipe, dryingClass, workshop, Instant.now());
    }

    /** @param when heure de la preparation, parametre pour que l'etat soit verifiable */
    @Transactional
    public PaletteMix record(String name, Rgb color, String recipe, DryingClass dryingClass,
                             Workshop workshop, Instant when) {
        return repository.save(new PaletteMix(name, color.toHex(), recipe, dryingClass, when,
                workshop.temperatureCelsius(), workshop.relativeHumidity(), workshop.ventilation()));
    }

    @Transactional
    public void forget(PaletteMix mix) {
        repository.delete(mix);
    }

    /** Efface les melanges qui ont pris : la palette se nettoie aussi. */
    @Transactional
    public int forgetSpent() {
        return forgetSpent(Instant.now());
    }

    /** @param now instant de reference, parametre pour que le menage soit verifiable */
    @Transactional
    public int forgetSpent(Instant now) {
        List<PaletteMix> spent = states(now).stream()
                .filter(MixState::isSpent).map(MixState::mix).toList();
        repository.deleteAll(spent);
        return spent.size();
    }

    public List<MixState> states() {
        return states(Instant.now());
    }

    /** @param now instant de reference, parametre pour que l'etat soit verifiable */
    public List<MixState> states(Instant now) {
        return findAll().stream().map(mix -> state(mix, now)).toList();
    }

    public MixState state(PaletteMix mix, Instant now) {
        DryingEstimate estimate = dryingTimeService.estimate(new DryingContext(
                mix.getDryingClass(), Medium.NONE, 0, ON_THE_PALETTE,
                new Workshop(mix.getTemperature(), mix.getHumidity(), mix.getVentilation())));

        Duration age = Duration.between(mix.getMixedAt(), now);
        if (age.isNegative()) {
            age = Duration.ZERO;
        }
        return new MixState(mix, remaining(estimate.openTime(), age), remaining(estimate.touchDry(), age));
    }

    private static Duration remaining(Duration milestone, Duration age) {
        Duration left = milestone.minus(age);
        return left.isNegative() ? Duration.ZERO : left;
    }
}
