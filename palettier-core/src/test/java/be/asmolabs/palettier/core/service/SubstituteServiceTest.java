package be.asmolabs.palettier.core.service;

import static org.assertj.core.api.Assertions.assertThat;

import be.asmolabs.palettier.core.domain.OilPaint;
import be.asmolabs.palettier.core.service.SubstituteService.Missing;
import java.util.List;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.transaction.annotation.Transactional;

@SpringBootTest
@Transactional
class SubstituteServiceTest {

    @Autowired
    private PaintCatalogService catalog;

    @Autowired
    private SubstituteService substitutes;

    private OilPaint byName(String name) {
        return catalog.search(name).stream()
                .filter(paint -> paint.getName().equals(name))
                .findFirst().orElseThrow();
    }

    @Test
    @DisplayName("un tube possede ne figure pas dans les manquants")
    void whatIsOwnedIsNotMissing() {
        catalog.declareNothingOwned();
        OilPaint umber = byName("Burnt Umber");
        catalog.setOwned(umber, true);

        assertThat(substitutes.missingAmong(List.of(byName("Burnt Umber")))).isEmpty();
    }

    @Test
    @DisplayName("un tube manquant se voit proposer le plus proche de l'etagere")
    void amissingPaintGetsTheClosestOwnedOne() {
        catalog.declareNothingOwned();
        // On ne possede que des terres : le remplacant d'une terre doit en etre une.
        catalog.setOwned(byName("Raw Umber"), true);
        catalog.setOwned(byName("Titanium White"), true);

        List<Missing> missing = substitutes.missingAmong(List.of(byName("Burnt Umber")));

        assertThat(missing).hasSize(1);
        Missing gap = missing.getFirst();
        assertThat(gap.paint().getName()).isEqualTo("Burnt Umber");
        assertThat(gap.nearest()).isPresent();
        assertThat(gap.nearest().get().getName())
                .as("le blanc est bien plus loin qu'une terre")
                .isEqualTo("Raw Umber");
        assertThat(gap.verdict()).contains("Raw Umber");
    }

    @Test
    @DisplayName("l'etagere vide se dit, elle ne provoque pas d'erreur")
    void anEmptyShelfIsStated() {
        catalog.declareNothingOwned();

        List<Missing> missing = substitutes.missingAmong(List.of(byName("Burnt Umber")));

        assertThat(missing).hasSize(1);
        assertThat(missing.getFirst().nearest()).isEmpty();
        assertThat(missing.getFirst().isComfortable()).isFalse();
        assertThat(missing.getFirst().verdict()).contains("Rien sur l'etagere");
    }
}
