package be.asmolabs.palettier.core.service;

import static org.assertj.core.api.Assertions.assertThat;

import be.asmolabs.palettier.core.catalog.CatalogLoader;
import be.asmolabs.palettier.core.catalog.PigmentIndex;
import be.asmolabs.palettier.core.color.Rgb;
import be.asmolabs.palettier.core.domain.OilPaint;
import java.util.List;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

/**
 * La recherche inverse explore toutes les paires du catalogue : son cout croit au
 * carre du nombre de tubes. Ce test fixe la borne au-dela de laquelle l'appel doit
 * imperativement quitter le fil d'affichage.
 */
class SuggestionPerformanceTest {

    private final ColorMixService service = new ColorMixService();
    private final List<OilPaint> catalog = new CatalogLoader(new PigmentIndex()).load();

    @Test
    @DisplayName("la recherche sur le catalogue complet reste sous la dizaine de secondes")
    void fullCatalogSearchCompletes() {
        long start = System.nanoTime();
        var suggestions = service.suggestMixes(Rgb.ofHex("#6B5A42"), catalog, 6);
        long millis = (System.nanoTime() - start) / 1_000_000;

        System.out.printf("recherche inverse : %d tubes, %d ms%n", catalog.size(), millis);
        assertThat(suggestions).hasSize(6);
        assertThat(millis).isLessThan(10_000);
    }
}
