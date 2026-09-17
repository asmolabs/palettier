package be.asmolabs.palettier.core.service;

import be.asmolabs.palettier.core.domain.LayerThickness;
import be.asmolabs.palettier.core.domain.Project;
import be.asmolabs.palettier.core.domain.ProjectLayer;
import be.asmolabs.palettier.core.domain.ProjectZone;
import be.asmolabs.palettier.core.domain.Recipe;
import be.asmolabs.palettier.core.domain.RecipeStep;
import be.asmolabs.palettier.core.domain.Technique;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import org.springframework.stereotype.Service;

/**
 * La regle du gras sur maigre, verifiee sur l'ordre des couches.
 *
 * <p>Une couche maigre posee sur une couche grasse seche plus vite qu'elle. Le dessous
 * continue de bouger quand le dessus a pris : la couche superieure tire, et craquele --
 * des mois plus tard, sur une piece finie. C'est la seule faute de cet atelier qui ne se
 * rattrape pas.</p>
 *
 * <p>La verification est volontairement silencieuse sur ce qui se pratique tous les
 * jours. Un glacis tres dilue sur un aplat de base est plus maigre que lui, et pourtant
 * personne n'a jamais fait craqueler une figurine ainsi : le glacis est un voile, il n'a
 * pas de quoi tirer. L'alerte ne se declenche donc que lorsque les deux conditions sont
 * reunies -- un ecart de gras franc, et une couche qui n'est pas plus fine que celle
 * qu'elle recouvre.</p>
 *
 * <p>Sur un projet, le gras de chaque couche se lit dans sa technique, qui porte deja
 * son medium et sa proportion usuelle : rien n'est demande au peintre, c'est la technique
 * qu'il a choisie qui parle. Sur une recette, ou le medium, la dilution et l'epaisseur
 * sont saisis un par un, la verification porte sur ces valeurs-la et non sur des
 * moyennes -- c'est le meme controle, en plus exact.</p>
 */
@Service
public class FatOverLeanService {

    /**
     * Ecart de gras a partir duquel l'empilement merite d'etre signale.
     *
     * <p>Sous ce seuil, on est dans la variation normale entre deux jus plus ou moins
     * dilues. Au-dela, on passe de la peinture a peu pres pure a un jus d'essence.</p>
     */
    private static final double SIGNIFICANT_DROP = 0.40;

    /**
     * Un empilement a risque.
     *
     * @param where zone du projet, ou nom de la recette
     * @param under couche du dessous, la plus grasse
     * @param over  couche posee dessus, plus maigre
     */
    public record Risk(String where, String under, String over, double drop, String explanation) {
    }

    /** Une couche reduite a ce qui decide du risque : son gras et son epaisseur. */
    private record Coat(String label, String role, double fatness, LayerThickness thickness) {
    }

    /** Les empilements a risque du projet, zone par zone, dans l'ordre des couches. */
    public List<Risk> inspect(Project project) {
        List<Risk> risks = new ArrayList<>();
        for (ProjectZone zone : project.getZones()) {
            risks.addAll(inspect(zone));
        }
        return List.copyOf(risks);
    }

    /**
     * Les empilements a risque d'une recette.
     *
     * <p>Une recette enonce son medium, sa dilution et son epaisseur pour chaque etape :
     * la verification s'appuie dessus, sans rien deduire. C'est le controle le plus sur
     * des deux, et c'est logique -- une recette est ecrite pour etre suivie telle quelle.</p>
     */
    public List<Risk> inspect(Recipe recipe) {
        List<Coat> coats = new ArrayList<>();
        List<RecipeStep> steps = recipe.getSteps();
        for (int i = 0; i < steps.size(); i++) {
            RecipeStep step = steps.get(i);
            coats.add(new Coat(step.getTechnique().label(), "Etape " + (i + 1),
                    step.getMedium().fatness(step.getMediumRatio()), step.getThickness()));
        }
        return inspect(recipe.getName(), coats);
    }

    private List<Risk> inspect(ProjectZone zone) {
        List<Coat> coats = zone.getLayers().stream().map(FatOverLeanService::coatOf).toList();
        return inspect(zone.getName(), coats);
    }

    /** Le controle lui-meme, une fois les couches reduites a ce qui compte. */
    private static List<Risk> inspect(String where, List<Coat> coats) {
        List<Risk> risks = new ArrayList<>();
        for (int i = 1; i < coats.size(); i++) {
            compare(where, coats.get(i - 1), coats.get(i)).ifPresent(risks::add);
        }
        return List.copyOf(risks);
    }

    private static Optional<Risk> compare(String where, Coat under, Coat over) {
        double drop = under.fatness() - over.fatness();
        if (drop < SIGNIFICANT_DROP) {
            return Optional.empty();
        }
        // Un voile ne tire pas sur ce qu'il recouvre : seule une couche au moins aussi
        // chargee que celle du dessous pose un probleme.
        if (over.thickness().compareTo(under.thickness()) < 0) {
            return Optional.empty();
        }
        return Optional.of(new Risk(where, under.role(), over.role(), drop,
                explain(under.label(), over.label())));
    }

    private static Coat coatOf(ProjectLayer layer) {
        Technique technique = Technique.byLabel(layer.getTechnique(), Technique.GLAZE);
        return new Coat(technique.label(), layer.getRole(), fatness(technique),
                technique.typicalThickness());
    }

    private static double fatness(Technique technique) {
        return technique.defaultMedium().fatness(technique.defaultRatio());
    }

    private static String explain(String below, String above) {
        return ("%s est plus maigre que %s, et pas plus fine. Posee dessus, elle sechera avant "
                + "elle et tirera dessus en vieillissant. Faites l'inverse, ou attendez le "
                + "sechage a coeur de la couche du dessous et allegez la main.")
                .formatted(above, below);
    }

    /** Comparaison brute de deux techniques, pour l'ecran qui veut l'expliquer. */
    public double fatnessOf(String technique) {
        return fatness(Technique.byLabel(technique, Technique.GLAZE));
    }

    /** Vrai si l'epaisseur usuelle de la technique en fait un simple voile. */
    public boolean isVeil(String technique) {
        return Technique.byLabel(technique, Technique.GLAZE).typicalThickness() == LayerThickness.GLAZE;
    }
}
