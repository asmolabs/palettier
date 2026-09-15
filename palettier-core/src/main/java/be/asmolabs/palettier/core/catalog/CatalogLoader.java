package be.asmolabs.palettier.core.catalog;

import be.asmolabs.palettier.core.domain.OilPaint;
import be.asmolabs.palettier.core.domain.Opacity;
import java.io.IOException;
import java.io.UncheckedIOException;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Set;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.core.io.Resource;
import org.springframework.core.io.support.PathMatchingResourcePatternResolver;
import org.springframework.stereotype.Component;

/**
 * Construit le catalogue a partir des fichiers de {@code catalog/brands/}.
 *
 * <p>Une gamme = un fichier. Ajouter une marque ne demande donc aucune recompilation
 * du code metier, seulement un fichier de plus dans les ressources.</p>
 *
 * <p>Seules les colonnes {@code brand}, {@code name} et {@code pigments} sont
 * obligatoires : l'opacite et la teinte sont deduites des pigments quand le fichier
 * ne les precise pas, et les tubes ainsi completes sont marques comme approximatifs.</p>
 */
@Component
public class CatalogLoader {

    private static final Logger log = LoggerFactory.getLogger(CatalogLoader.class);
    private static final String LOCATION = "classpath:catalog/brands/*.csv";

    private final PigmentIndex pigments;

    public CatalogLoader(PigmentIndex pigments) {
        this.pigments = pigments;
    }

    public List<OilPaint> load() {
        List<OilPaint> paints = new ArrayList<>();
        Set<String> citedPigments = new LinkedHashSet<>();

        for (Resource resource : brandFiles()) {
            int before = paints.size();
            for (Csv.Row row : Csv.read(resource)) {
                OilPaint paint = toPaint(row);
                citedPigments.addAll(paint.getPigments());
                paints.add(paint);
            }
            log.info("{} : {} huiles", resource.getFilename(), paints.size() - before);
        }

        long twoConstant = paints.stream().filter(paint -> paint.getTintHex() != null).count();
        log.info("{} huiles au total, dont {} avec teinte diluee connue (melange a deux constantes)",
                paints.size(), twoConstant);

        List<String> unknown = pigments.unknownAmong(citedPigments);
        if (!unknown.isEmpty()) {
            log.warn("{} pigments cites par le catalogue mais absents de pigments.csv : {}",
                    unknown.size(), String.join(", ", unknown));
        }
        return paints;
    }

    private Resource[] brandFiles() {
        try {
            Resource[] resources = new PathMatchingResourcePatternResolver().getResources(LOCATION);
            Arrays.sort(resources, (a, b) -> String.valueOf(a.getFilename()).compareTo(String.valueOf(b.getFilename())));
            return resources;
        } catch (IOException e) {
            throw new UncheckedIOException("Lecture impossible du catalogue " + LOCATION, e);
        }
    }

    private OilPaint toPaint(Csv.Row row) {
        Set<String> pigmentCodes = parsePigments(row.get("pigments"));
        if (pigmentCodes.isEmpty()) {
            throw new IllegalStateException("Aucun pigment declare en " + row.location());
        }

        String declaredHex = row.getOrDefault("hex", "");
        boolean derived = declaredHex.isBlank();
        String hex = derived ? pigments.colorFor(pigmentCodes).toHex() : declaredHex;

        Opacity opacity = row.getOrDefault("opacity", "").isBlank()
                ? pigments.opacityFor(pigmentCodes)
                : Opacity.valueOf(row.get("opacity"));

        String declaredTint = row.getOrDefault("tint", "");
        String tint = declaredTint.isBlank()
                ? pigments.tintFor(pigmentCodes, be.asmolabs.palettier.core.color.Rgb.ofHex(hex))
                        .map(be.asmolabs.palettier.core.color.Rgb::toHex).orElse(null)
                : declaredTint;

        OilPaint paint = new OilPaint(
                row.get("brand"),
                row.get("name"),
                row.getOrDefault("code", ""),
                hex,
                opacity,
                pigments.dryingClassFor(pigmentCodes),
                pigments.tintingStrengthFor(pigmentCodes),
                pigmentCodes);
        paint.setColorDerived(derived);
        paint.setTintHex(tint);
        return paint;
    }

    /** Les pigments sont listes separes par des espaces : "PBr7 PBk9". */
    private static Set<String> parsePigments(String cell) {
        return Arrays.stream(cell.split("\\s+"))
                .map(String::trim)
                .filter(code -> !code.isEmpty())
                .collect(LinkedHashSet::new, Set::add, Set::addAll);
    }
}
