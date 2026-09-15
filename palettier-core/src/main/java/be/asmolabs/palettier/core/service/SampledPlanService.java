package be.asmolabs.palettier.core.service;

import be.asmolabs.palettier.core.color.Colors;
import be.asmolabs.palettier.core.color.Rgb;
import be.asmolabs.palettier.core.domain.OilPaint;
import be.asmolabs.palettier.core.domain.Palette;
import be.asmolabs.palettier.core.domain.Technique;
import be.asmolabs.palettier.core.plan.PaintingPlan;
import be.asmolabs.palettier.core.service.MixModels.MixSuggestion;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import org.springframework.stereotype.Service;

/**
 * Batit un plan de peinture a partir de couleurs relevees sur une reference.
 *
 * <p>C'est le chemin sans assistant : on photographie ce qu'on veut obtenir, on preleve
 * plusieurs points par partie du sujet -- le creux de l'orbite, la joue, l'arete du nez --
 * et l'application en tire le plan. Rien n'est devine : chaque couche vient d'une mesure.</p>
 *
 * <p>Les roles sont deduits de la clarte des points, et non de l'ordre de prelevement :
 * on ne releve pas une reference dans l'ordre, on releve ce qu'on voit. Le point median
 * devient la base, les plus sombres les ombres, les plus clairs les lumieres.</p>
 */
@Service
public class SampledPlanService {

    private final ColorMixService mixer;

    public SampledPlanService(ColorMixService mixer) {
        this.mixer = mixer;
    }

    /**
     * Un groupe de points releves sur une meme partie du sujet.
     *
     * @param name     ce que la partie represente : "peau", "cape", "cuir des sangles"
     * @param material precision libre sur la matiere, eventuellement vide
     * @param samples  couleurs relevees, dans n'importe quel ordre
     */
    public record ColourGroup(String name, String material, List<Rgb> samples) {

        public ColourGroup {
            samples = List.copyOf(samples);
        }
    }

    /**
     * @param maxPaints nombre maximal de tubes par melange
     * @throws IllegalArgumentException si aucun groupe ne porte de releve
     */
    public PaintingPlan plan(String subject, Palette palette, List<ColourGroup> groups, int maxPaints) {
        List<ColourGroup> usable = groups.stream().filter(g -> !g.samples().isEmpty()).toList();
        if (usable.isEmpty()) {
            throw new IllegalArgumentException("Aucun point releve : ajoutez au moins une couleur a un groupe.");
        }

        List<PaintingPlan.Zone> zones = usable.stream()
                .map(group -> zone(group, palette, maxPaints))
                .toList();

        return new PaintingPlan(subject, palette == null ? "sans palette" : palette.getName(),
                "Plan etabli a partir de %d points releves sur la reference.".formatted(
                        usable.stream().mapToInt(g -> g.samples().size()).sum()),
                zones);
    }

    private PaintingPlan.Zone zone(ColourGroup group, Palette palette, int maxPaints) {
        List<Rgb> ordered = group.samples().stream()
                .sorted(Comparator.comparingDouble(Rgb::relativeLuminance))
                .toList();

        // Le point median tient la valeur locale : c'est la base. Ce qui est plus sombre
        // descend dans les ombres, ce qui est plus clair monte dans les lumieres.
        int baseIndex = ordered.size() / 2;

        List<PaintingPlan.Layer> shadows = new ArrayList<>();
        for (int i = baseIndex - 1, rank = 1; i >= 0; i--, rank++) {
            shadows.add(layer("Ombre " + rank, ordered.get(i), Technique.GLAZE, palette, maxPaints));
        }

        List<PaintingPlan.Layer> highlights = new ArrayList<>();
        for (int i = baseIndex + 1, rank = 1; i < ordered.size(); i++, rank++) {
            highlights.add(layer("Lumiere " + rank, ordered.get(i), Technique.HIGHLIGHT, palette, maxPaints));
        }

        return new PaintingPlan.Zone(group.name(), group.material(), noteFor(ordered.size()),
                layer("Base", ordered.get(baseIndex), Technique.BASE_LAYER, palette, maxPaints),
                List.copyOf(shadows), List.copyOf(highlights));
    }

    /** Un mot sur ce que le nombre de points permet, ou ne permet pas. */
    private static String noteFor(int sampleCount) {
        return switch (sampleCount) {
            case 1 -> "Un seul point releve : la valeur locale, sans modele. Relevez une ombre et "
                    + "une lumiere pour obtenir un degrade.";
            case 2 -> "Deux points : une base et un ecart. Un troisieme de l'autre cote donnerait "
                    + "un vrai degrade.";
            default -> "%d points releves sur la reference.".formatted(sampleCount);
        };
    }

    private PaintingPlan.Layer layer(String role, Rgb target, Technique technique,
                                     Palette palette, int maxPaints) {
        List<OilPaint> paints = palette == null ? List.of() : palette.getPaints();
        if (paints.isEmpty()) {
            return new PaintingPlan.Layer(role, target, technique.label(), "", null, target, 0);
        }
        List<MixSuggestion> found = mixer.suggestMixes(target, paints, 1, maxPaints);
        if (found.isEmpty()) {
            return new PaintingPlan.Layer(role, target, technique.label(), "", null, target, 0);
        }
        MixSuggestion best = found.getFirst();
        return new PaintingPlan.Layer(role, target, technique.label(), "",
                best, best.color(), Colors.deltaE2000(target, best.color()));
    }
}
