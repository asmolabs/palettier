package be.asmolabs.palettier.ai;

import be.asmolabs.palettier.core.color.Rgb;
import be.asmolabs.palettier.core.domain.Palette;
import be.asmolabs.palettier.core.service.MixModels.PaintMatch;
import be.asmolabs.palettier.core.service.PaintCatalogService;
import be.asmolabs.palettier.core.service.PaletteService;
import java.util.List;
import org.springframework.ai.tool.annotation.Tool;
import org.springframework.stereotype.Component;

/**
 * Ce que le modele peut demander a l'application pendant qu'il reflechit.
 *
 * <p>Tous ces outils repondent par des mesures issues du module metier. Aucun ne laisse
 * le modele produire un chiffre : c'est la seule facon d'obtenir un conseil ou les
 * couleurs annoncees correspondent a quelque chose de reel.</p>
 */
@Component
public class OilPainterTools {

    private final PaintCatalogService catalog;
    private final PaletteService palettes;

    OilPainterTools(PaintCatalogService catalog, PaletteService palettes) {
        this.catalog = catalog;
        this.palettes = palettes;
    }

    @Tool(description = "Liste les palettes que le peintre a composees, avec le nombre de tubes de chacune.")
    public List<String> listerPalettes() {
        return palettes.findAll().stream()
                .map(palette -> "%s (%d tubes, %s)".formatted(
                        palette.getName(), palette.getPaints().size(), palette.getPurpose()))
                .toList();
    }

    @Tool(description = "Donne le contenu exact d'une palette : nom, marque et couleur hexadecimale de chaque tube. "
            + "A consulter avant de proposer des couleurs, pour ne viser que ce qui est atteignable.")
    public List<String> contenuPalette(String nomDeLaPalette) {
        return palettes.findAll().stream()
                .filter(palette -> palette.getName().equalsIgnoreCase(nomDeLaPalette))
                .findFirst()
                .map(Palette::getPaints)
                .orElse(List.of())
                .stream()
                .map(paint -> "%s - %s : %s, pigments %s".formatted(
                        paint.getBrand(), paint.getName(), paint.getHexColor(),
                        String.join(" ", paint.getPigments())))
                .toList();
    }

    @Tool(description = "Trouve les tubes du catalogue les plus proches d'une couleur hexadecimale, "
            + "avec l'ecart percu CIEDE2000 de chacun.")
    public List<String> tubesLesPlusProches(String couleurHexadecimale) {
        try {
            return catalog.findClosest(Rgb.ofHex(couleurHexadecimale), false, 5).stream()
                    .map(this::describe)
                    .toList();
        } catch (IllegalArgumentException e) {
            return List.of("Couleur invalide, format attendu #RRGGBB");
        }
    }

    private String describe(PaintMatch match) {
        return "%s : %s, ecart %.1f (%s)".formatted(
                match.paint().displayName(), match.paint().getHexColor(),
                match.deltaE(), match.verdict());
    }
}
