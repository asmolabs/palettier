package be.asmolabs.palettier.core.service;

import be.asmolabs.palettier.core.domain.OilPaint;
import be.asmolabs.palettier.core.domain.Project;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import org.springframework.stereotype.Service;

/**
 * Que faire des tubes qu'un projet reclame et qu'on n'a pas.
 *
 * <p>L'inventaire savait dire qu'un tube manquait ; il ne servait a rien d'autre. Or la
 * question du peintre n'est pas "que me manque-t-il" -- il le sait -- mais "avec quoi je
 * m'en sors ce soir". Le moteur qui repond existe deja : c'est celui qui classe les
 * huiles par ecart percu. Il suffisait de le brancher sur l'etagere.</p>
 *
 * <p>Ce n'est pas une liste de courses. Un tube manquant dont on possede l'equivalent a
 * un ecart invisible n'a pas besoin d'etre achete, et c'est une information plus utile
 * que son absence.</p>
 */
@Service
public class SubstituteService {

    /** Au-dela, l'ecart se voit cote a cote et le remplacement ne va plus de soi. */
    private static final double VISIBLE_GAP = 5.0;

    private final PaintCatalogService catalog;

    public SubstituteService(PaintCatalogService catalog) {
        this.catalog = catalog;
    }

    /**
     * Un tube absent de l'etagere, et le plus proche de ceux qu'on possede.
     *
     * @param nearest    le tube de rechange, ou vide si l'etagere est vide
     * @param deltaE     ecart percu entre le tube manquant et son remplacant
     */
    public record Missing(OilPaint paint, Optional<OilPaint> nearest, double deltaE) {

        /** Vrai quand le remplacement ne se verra pas sur la piece. */
        public boolean isComfortable() {
            return nearest.isPresent() && deltaE < VISIBLE_GAP;
        }

        public String verdict() {
            if (nearest.isEmpty()) {
                return "Rien sur l'etagere pour le remplacer.";
            }
            if (deltaE < 2) {
                return "%s le remplace sans que cela se voie.".formatted(nearest.get().displayName());
            }
            if (deltaE < VISIBLE_GAP) {
                return "%s en approche : l'ecart ne se verra pas sur la piece."
                        .formatted(nearest.get().displayName());
            }
            if (deltaE < 12) {
                return "%s est ce qui s'en rapproche le plus, mais l'ecart se voit : a rattraper au glacis."
                        .formatted(nearest.get().displayName());
            }
            return "Rien d'approchant sur l'etagere : celui-la, il faut l'acheter.";
        }
    }

    /** Les tubes du projet qui ne sont pas sur l'etagere, avec leur meilleur remplacant. */
    public List<Missing> missingFrom(Project project) {
        return missingAmong(project.effectivePaints());
    }

    /** Idem pour une selection quelconque de tubes. */
    public List<Missing> missingAmong(List<OilPaint> wanted) {
        List<OilPaint> owned = catalog.findInStock();
        List<Missing> missing = new ArrayList<>();

        for (OilPaint paint : wanted) {
            if (paint.isInStock()) {
                continue;
            }
            // Le tube manquant ne doit evidemment pas se proposer lui-meme, et un tube
            // identique d'une autre gamme reste un remplacant legitime.
            Optional<OilPaint> nearest = owned.stream()
                    .filter(candidate -> !candidate.getId().equals(paint.getId()))
                    .min((a, b) -> Double.compare(gap(paint, a), gap(paint, b)));
            missing.add(new Missing(paint, nearest,
                    nearest.map(candidate -> gap(paint, candidate)).orElse(Double.MAX_VALUE)));
        }
        return List.copyOf(missing);
    }

    private static double gap(OilPaint wanted, OilPaint candidate) {
        return be.asmolabs.palettier.core.color.Colors.deltaE2000(wanted.color(), candidate.color());
    }
}
