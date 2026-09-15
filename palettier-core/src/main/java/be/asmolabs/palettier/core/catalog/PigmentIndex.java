package be.asmolabs.palettier.core.catalog;

import be.asmolabs.palettier.core.color.Colorant;
import be.asmolabs.palettier.core.color.Colors;
import be.asmolabs.palettier.core.color.Rgb;
import be.asmolabs.palettier.core.domain.DryingClass;
import be.asmolabs.palettier.core.domain.Opacity;
import be.asmolabs.palettier.core.domain.Pigment;
import java.util.Collection;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.Set;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.core.io.ClassPathResource;
import org.springframework.stereotype.Component;

/**
 * Table des pigments, chargee depuis {@code catalog/pigments.csv}.
 *
 * <p>Sert a deduire les proprietes d'une huile a partir de sa composition, plutot
 * que de les saisir tube par tube : le sechage d'un melange est celui de son pigment
 * le plus lent, le pouvoir colorant celui de son pigment le plus fort.</p>
 */
@Component
public class PigmentIndex {

    private static final Logger log = LoggerFactory.getLogger(PigmentIndex.class);
    private static final String LOCATION = "catalog/pigments.csv";

    private final Map<String, Pigment> byCode = new LinkedHashMap<>();

    public PigmentIndex() {
        Csv.read(new ClassPathResource(LOCATION)).forEach(row -> {
            Pigment pigment = new Pigment(
                    row.get("code"),
                    row.get("name"),
                    Rgb.ofHex(row.get("hex")),
                    Opacity.valueOf(row.get("opacity")),
                    DryingClass.valueOf(row.get("drying")),
                    Double.parseDouble(row.get("tinting")),
                    row.getOrDefault("tint", "").isBlank() ? null : Rgb.ofHex(row.get("tint")));
            byCode.put(normalise(pigment.code()), pigment);
        });
        long withTint = byCode.values().stream().filter(pigment -> pigment.tintColor().isPresent()).count();
        log.info("{} pigments references, dont {} avec teinte diluee relevee", byCode.size(), withTint);
    }

    public Optional<Pigment> find(String code) {
        return Optional.ofNullable(byCode.get(normalise(code)));
    }

    public Collection<Pigment> all() {
        return byCode.values();
    }

    /** Codes cites par le catalogue mais absents de la table : liste de travail. */
    public List<String> unknownAmong(Collection<String> codes) {
        return codes.stream().filter(code -> find(code).isEmpty()).distinct().sorted().toList();
    }

    // --- Proprietes deduites d'une composition -----------------------------

    private List<Pigment> resolve(Set<String> codes) {
        return codes.stream().map(code -> find(code).orElseGet(() -> Pigment.unknown(code))).toList();
    }

    /** Le pigment le plus lent commande : une trace de cadmium suffit a tout ralentir. */
    public DryingClass dryingClassFor(Set<String> codes) {
        return resolve(codes).stream()
                .map(Pigment::dryingClass)
                .reduce(DryingClass.FAST, DryingClass::slowest);
    }

    /** Le pigment le plus colorant domine le melange. */
    public double tintingStrengthFor(Set<String> codes) {
        return resolve(codes).stream()
                .mapToDouble(Pigment::tintingStrength)
                .max()
                .orElse(0.5);
    }

    /** A defaut d'indication du fabricant, le pigment le plus couvrant donne le ton. */
    public Opacity opacityFor(Set<String> codes) {
        return resolve(codes).stream()
                .map(Pigment::opacity)
                .reduce(Opacity.TRANSPARENT, (a, b) -> a.compareTo(b) >= 0 ? a : b);
    }

    /**
     * Couleur approchee d'une huile dont la teinte n'est pas renseignee, obtenue en
     * melangeant ses pigments a parts egales. Correct pour un tube monopigmentaire,
     * approximatif des que le fabricant dose ses pigments, ce que personne ne publie.
     */
    public Rgb colorFor(Set<String> codes) {
        List<Pigment> pigments = resolve(codes);
        if (pigments.isEmpty()) {
            return new Rgb(0.5, 0.5, 0.5);
        }
        return Colors.mix(pigments.stream().map(Pigment::color).toList(),
                pigments.stream().map(Pigment::tintingStrength).toList());
    }

    /**
     * Teinte diluee estimee d'une huile, deduite de ses pigments.
     *
     * <p>La divergence entre ton de masse et teinte diluee est une propriete du pigment ;
     * le ton de masse, lui, est propre au tube. On ne recopie donc pas la teinte du
     * pigment telle quelle : on mesure de combien elle s'ecarte de ce que le modele a
     * constante unique aurait predit, et on reporte cet ecart sur la teinte predite pour
     * le tube. Un pigment sans teinte relevee n'apporte aucun ecart, et le tube garde
     * alors le comportement d'avant.</p>
     *
     * @return la teinte estimee, ou vide si aucun pigment du tube n'a de teinte relevee
     */
    public Optional<Rgb> tintFor(Set<String> codes, Rgb masstone) {
        List<Pigment> pigments = resolve(codes);
        if (pigments.stream().allMatch(pigment -> pigment.tintColor().isEmpty())) {
            return Optional.empty();
        }

        double[] gain = {0, 0, 0};
        double total = 0;
        for (Pigment pigment : pigments) {
            double weight = pigment.tintingStrength();
            double[] pigmentGain = divergenceOf(pigment);
            for (int channel = 0; channel < 3; channel++) {
                gain[channel] += weight * pigmentGain[channel];
            }
            total += weight;
        }

        Rgb predicted = Colorant.ofMasstone(masstone).tint(Colorant.REFERENCE_TINT_CONCENTRATION);
        double[] predictedLinear = {
                Colors.srgbToLinear(predicted.r()),
                Colors.srgbToLinear(predicted.g()),
                Colors.srgbToLinear(predicted.b())};

        return Optional.of(new Rgb(
                Colors.linearToSrgb(predictedLinear[0] * gain[0] / total),
                Colors.linearToSrgb(predictedLinear[1] * gain[1] / total),
                Colors.linearToSrgb(predictedLinear[2] * gain[2] / total)));
    }

    /**
     * De combien la teinte relevee d'un pigment s'ecarte de celle que le modele a
     * constante unique aurait predite, canal par canal. Vaut 1 partout quand le pigment
     * n'a pas de teinte relevee : aucun ecart, donc aucune correction.
     */
    private static double[] divergenceOf(Pigment pigment) {
        return pigment.tintColor()
                .map(observed -> {
                    Rgb predicted = Colorant.ofMasstone(pigment.color())
                            .tint(Colorant.REFERENCE_TINT_CONCENTRATION);
                    return new double[]{
                            ratio(observed.r(), predicted.r()),
                            ratio(observed.g(), predicted.g()),
                            ratio(observed.b(), predicted.b())};
                })
                .orElse(new double[]{1, 1, 1});
    }

    private static double ratio(double observed, double predicted) {
        return Colors.srgbToLinear(observed) / Math.max(Colors.srgbToLinear(predicted), 1e-5);
    }

    private static String normalise(String code) {
        return code.trim().toUpperCase().replace(" ", "");
    }
}
