package be.asmolabs.palettier.core.service;

import be.asmolabs.palettier.core.color.Colorant;
import be.asmolabs.palettier.core.color.Colors;
import be.asmolabs.palettier.core.color.Rgb;
import be.asmolabs.palettier.core.domain.DryingClass;
import be.asmolabs.palettier.core.domain.OilPaint;
import be.asmolabs.palettier.core.domain.Opacity;
import be.asmolabs.palettier.core.service.MixModels.MixComponent;
import be.asmolabs.palettier.core.service.MixModels.MixResult;
import be.asmolabs.palettier.core.service.MixModels.MixSuggestion;
import be.asmolabs.palettier.core.service.MixModels.PaintPart;
import java.util.ArrayList;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Set;
import org.springframework.stereotype.Service;

/**
 * Simule le melange de plusieurs huiles et propose des recettes pour atteindre une teinte.
 */
@Service
public class ColorMixService {

    /** Au-dela de ce nombre de tubes, le melange tourne au gris quoi qu'on fasse. */
    private static final int MUDDY_MIX_THRESHOLD = 4;

    /** Nombre de tubes retenu par defaut pour une proposition de melange. */
    private static final int DEFAULT_MAX_PAINTS = 3;

    public MixResult mix(List<PaintPart> parts) {
        if (parts.isEmpty()) {
            throw new IllegalArgumentException("Un melange demande au moins une huile");
        }

        double totalParts = parts.stream().mapToDouble(PaintPart::parts).sum();
        double totalPigment = parts.stream()
                .mapToDouble(p -> p.parts() * p.paint().getTintingStrength())
                .sum();

        // Chaque tube apporte ses propres constantes d'absorption et de diffusion : celles
        // d'un vrai Kubelka-Munk a deux constantes quand sa teinte diluee est connue, celles
        // du modele a constante unique sinon.
        List<Colorant> colorants = parts.stream().map(p -> p.paint().colorant()).toList();
        List<Double> weights = parts.stream()
                .map(p -> p.parts() * p.paint().getTintingStrength())
                .toList();

        Rgb mixed = Colorant.mix(colorants, weights);

        List<MixComponent> components = parts.stream()
                .map(p -> new MixComponent(p.paint(), p.parts(),
                        p.parts() / totalParts,
                        p.parts() * p.paint().getTintingStrength() / totalPigment))
                .toList();

        DryingClass drying = parts.stream()
                .map(p -> p.paint().getDryingClass())
                .reduce(DryingClass.FAST, DryingClass::slowest);

        Set<String> pigments = new LinkedHashSet<>();
        parts.forEach(p -> pigments.addAll(p.paint().getPigments()));

        return new MixResult(mixed, components, drying, pigments, warningsFor(components, drying));
    }

    private List<String> warningsFor(List<MixComponent> components, DryingClass drying) {
        List<String> warnings = new ArrayList<>();

        if (components.size() >= MUDDY_MIX_THRESHOLD) {
            warnings.add("%d huiles dans le melange : le resultat va tendre vers le gris. Deux ou trois suffisent presque toujours."
                    .formatted(components.size()));
        }

        components.stream()
                .filter(c -> c.pigmentShare() > 0.6 && c.volumeShare() < 0.25)
                .forEach(c -> warnings.add(
                        "%s occupe %.0f %% du volume mais %.0f %% de la couleur finale : dosez-la a la pointe du pinceau."
                                .formatted(c.paint().getName(), c.volumeShare() * 100, c.pigmentShare() * 100)));

        boolean mixesOpacities = components.stream().anyMatch(c -> c.paint().getOpacity() == Opacity.TRANSPARENT)
                && components.stream().anyMatch(c -> c.paint().getOpacity() == Opacity.OPAQUE);
        if (mixesOpacities) {
            warnings.add("Melange d'une transparente et d'une opaque : le glacis perdra sa profondeur.");
        }

        if (drying.compareTo(DryingClass.SLOW) >= 0) {
            warnings.add("Sechage %s impose par le pigment le plus lent : prevoyez la couche suivante en consequence."
                    .formatted(drying.label().toLowerCase()));
        }

        Set<String> pigments = new LinkedHashSet<>();
        components.forEach(c -> pigments.addAll(c.paint().getPigments()));
        if (pigments.size() > 4) {
            warnings.add("%d pigments differents en presence : au-dela de quatre, la teinte devient difficile a reproduire."
                    .formatted(pigments.size()));
        }

        return List.copyOf(warnings);
    }

    /**
     * Cherche comment obtenir une couleur cible avec les huiles fournies.
     *
     * <p>Voir {@link MixSearch} pour la strategie. Les dosages proposes sont des
     * rapports d'entiers simples : l'ecart annonce est donc celui du melange qu'on peut
     * reellement faire, pas celui d'un optimum theorique.</p>
     *
     * @param target     couleur visee
     * @param candidates huiles disponibles, typiquement celles d'une palette
     * @param maxResults nombre de propositions a renvoyer
     */
    public List<MixSuggestion> suggestMixes(Rgb target, List<OilPaint> candidates, int maxResults) {
        return suggestMixes(target, candidates, maxResults, DEFAULT_MAX_PAINTS);
    }

    /**
     * @param maxPaints nombre maximal de tubes par proposition, de 1 a 3. Au-dela de
     *                  trois, un melange devient impossible a reproduire d'une seance
     *                  a l'autre et tourne au gris.
     */
    public List<MixSuggestion> suggestMixes(Rgb target, List<OilPaint> candidates,
                                            int maxResults, int maxPaints) {
        if (candidates.isEmpty() || maxResults <= 0) {
            return List.of();
        }
        return new MixSearch(target, candidates)
                .search(Math.clamp(maxPaints, 1, 3), maxResults);
    }
}
