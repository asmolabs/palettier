package be.asmolabs.palettier.core.service;

import be.asmolabs.palettier.core.domain.LayerThickness;
import be.asmolabs.palettier.core.domain.Project;
import be.asmolabs.palettier.core.domain.ProjectLayer;
import be.asmolabs.palettier.core.domain.ProjectZone;
import be.asmolabs.palettier.core.domain.Technique;
import java.util.ArrayList;
import java.util.List;
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
 * <p>Le gras de chaque couche se lit dans sa technique, qui porte deja son medium et sa
 * proportion usuelle. Rien n'est demande au peintre : c'est la technique qu'il a choisie
 * qui parle.</p>
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
     * @param under couche du dessous, la plus grasse
     * @param over  couche posee dessus, plus maigre
     */
    public record Risk(String zone, String under, String over, double drop, String explanation) {
    }

    /** Les empilements a risque du projet, zone par zone, dans l'ordre des couches. */
    public List<Risk> inspect(Project project) {
        List<Risk> risks = new ArrayList<>();
        for (ProjectZone zone : project.getZones()) {
            risks.addAll(inspect(zone));
        }
        return List.copyOf(risks);
    }

    private List<Risk> inspect(ProjectZone zone) {
        List<ProjectLayer> layers = zone.getLayers();
        List<Risk> risks = new ArrayList<>();

        for (int i = 1; i < layers.size(); i++) {
            ProjectLayer under = layers.get(i - 1);
            ProjectLayer over = layers.get(i);

            Technique below = Technique.byLabel(under.getTechnique(), Technique.GLAZE);
            Technique above = Technique.byLabel(over.getTechnique(), Technique.GLAZE);

            double drop = fatness(below) - fatness(above);
            if (drop < SIGNIFICANT_DROP) {
                continue;
            }
            // Un voile ne tire pas sur ce qu'il recouvre : seule une couche au moins
            // aussi chargee que celle du dessous pose un probleme.
            if (above.typicalThickness().compareTo(below.typicalThickness()) < 0) {
                continue;
            }
            risks.add(new Risk(zone.getName(), under.getRole(), over.getRole(), drop,
                    explain(below, above)));
        }
        return risks;
    }

    private static double fatness(Technique technique) {
        return technique.defaultMedium().fatness(technique.defaultRatio());
    }

    private static String explain(Technique below, Technique above) {
        return ("%s est plus maigre que %s, et pas plus fine. Posee dessus, elle sechera avant "
                + "elle et tirera dessus en vieillissant. Faites l'inverse, ou attendez le "
                + "sechage a coeur de la couche du dessous et allegez la main.")
                .formatted(above.label(), below.label());
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
