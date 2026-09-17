package be.asmolabs.palettier.core.service;

import be.asmolabs.palettier.core.color.Colors;
import be.asmolabs.palettier.core.color.Rgb;
import be.asmolabs.palettier.core.domain.Project;
import be.asmolabs.palettier.core.domain.ProjectLayer;
import be.asmolabs.palettier.core.domain.ProjectZone;
import be.asmolabs.palettier.core.image.ImagePalette;
import java.util.ArrayList;
import java.util.List;
import org.springframework.stereotype.Service;

/**
 * Ce qu'il y a sur la piece, compare a ce qui etait vise.
 *
 * <p>Le projet dit la couleur visee pour chaque couche, la pipette sait relever celles
 * d'une image : il ne manquait que de les mettre face a face. Le peintre voit alors non
 * seulement qu'il a rate, mais de combien et dans quelle direction -- ce qu'un oeil,
 * meme exerce, ne chiffre pas.</p>
 *
 * <p>Le sens de lecture est celui de la piece vers le plan, et non l'inverse. Partir des
 * couleurs visees pour chercher la plus proche sur la photo trouverait toujours quelque
 * chose : une photo contient des milliers de teintes. Partir de ce qui occupe reellement
 * la piece, et demander a quoi cela devait correspondre, se trompe moins -- et signale au
 * passage ce qui n'etait prevu nulle part.</p>
 *
 * <p>La mesure se fait sur le fichier d'origine du peintre, jamais sur la photo rangee
 * avec le projet : celle-ci est reduite et reencodee, et ses couleurs ont bouge. Meme
 * ainsi, une photo reste prise sous une lumiere quelconque -- l'ecart chiffre ici se lit
 * comme une tendance, pas comme un verdict.</p>
 */
@Service
public class ProgressCheckService {

    /** Au-dela de cet ecart, la teinte relevee ne correspond a rien du plan. */
    private static final double UNPLANNED = 12.0;

    /**
     * Une teinte relevee sur la piece, et la couche a laquelle elle repond.
     *
     * @param share part de l'image occupee par cette teinte
     * @param zone  zone du plan visee, ou {@code null} si rien n'y ressemble
     */
    public record Observed(Rgb measured, double share, String zone, String role,
                           Rgb target, double deltaE) {

        public boolean matchesPlan() {
            return zone != null && deltaE < UNPLANNED;
        }

        public String verdict() {
            if (!matchesPlan()) {
                return "Rien de prevu ne ressemble a cette teinte : appret, socle, ou une couche improvisee.";
            }
            if (deltaE < 2) {
                return "%s / %s : vous y etes.".formatted(zone, role);
            }
            if (deltaE < 5) {
                return "%s / %s : l'ecart ne se verra pas sur la piece.".formatted(zone, role);
            }
            return "%s / %s : l'ecart se voit, un glacis le rattraperait.".formatted(zone, role);
        }
    }

    /**
     * Compare les teintes d'une photo aux couleurs visees par le projet.
     *
     * @param original photo telle que le peintre l'a prise, non reduite
     * @param colours  nombre de teintes dominantes a relever
     */
    public List<Observed> compare(Project project, byte[] original, int colours) {
        List<ImagePalette.DominantColour> measured = ImagePalette.dominant(original, colours);
        List<Observed> observed = new ArrayList<>();

        for (ImagePalette.DominantColour dominant : measured) {
            Observed closest = null;
            for (ProjectZone zone : project.getZones()) {
                for (ProjectLayer layer : zone.getLayers()) {
                    double gap = Colors.deltaE2000(dominant.color(), layer.target());
                    if (closest == null || gap < closest.deltaE()) {
                        closest = new Observed(dominant.color(), dominant.share(),
                                zone.getName(), layer.getRole(), layer.target(), gap);
                    }
                }
            }
            observed.add(closest == null
                    ? new Observed(dominant.color(), dominant.share(), null, null, null, Double.MAX_VALUE)
                    : closest);
        }
        return List.copyOf(observed);
    }
}
