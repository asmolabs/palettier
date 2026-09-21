package be.asmolabs.palettier.core.backup;

import static org.assertj.core.api.Assertions.assertThat;

import be.asmolabs.palettier.core.domain.DryingClass;
import be.asmolabs.palettier.core.domain.Project;
import be.asmolabs.palettier.core.domain.Ventilation;
import be.asmolabs.palettier.core.service.PaintCatalogService;
import be.asmolabs.palettier.core.service.ProjectService;
import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Instant;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.transaction.annotation.Transactional;

/**
 * L'application Java relit-elle ce que le portage Kotlin ecrit ?
 *
 * <p>C'est ce qui rend la migration reversible. Tant que les deux applications parlent la
 * meme langue, revenir en arriere reste possible.</p>
 */
@SpringBootTest
@Transactional
class ReadKotlinArchiveTest {

    @Autowired private BackupService backup;
    @Autowired private ProjectService projects;
    @Autowired private PaintCatalogService catalog;

    @Test
    @DisplayName("une archive ecrite par le portage Kotlin se relit ici")
    void aKotlinArchiveIsReadable() throws Exception {
        // L'archive est produite par ExportForJavaTest, cote portage. Sans elle il n'y a
        // rien a verifier : on passe, plutot que d'echouer sur une absence d'artefact.
        Path archive = Path.of("../palettier-kmp/core-data/build/sortie-kotlin.zip");
        org.junit.jupiter.api.Assumptions.assumeTrue(Files.exists(archive),
                "archive Kotlin absente : lancez d'abord ./gradlew :core-data:jvmTest cote palettier-kmp");

        catalog.declareNothingOwned();
        BackupService.ImportReport report = backup.importFrom(archive);

        System.out.printf("RELU %d tubes mis a jour, %d projets, %d photos%n",
                report.paintsUpdated(), report.projectsAdded(), report.photos());

        assertThat(report.paintsUpdated()).isGreaterThan(600);
        assertThat(report.projectsAdded()).isEqualTo(1);
        assertThat(report.photos()).isEqualTo(1);

        Project project = projects.findAll().stream()
                .filter(p -> p.getName().equals("Grognard de reference"))
                .findFirst().orElseThrow();

        assertThat(project.getZones()).hasSize(1);
        assertThat(project.getZones().getFirst().getLayers()).hasSize(6);

        // Et la pose survit au trajet complet : Java, Kotlin, Java.
        var applied = project.getZones().getFirst().getLayers().stream()
                .filter(l -> l.isApplied()).findFirst().orElseThrow();
        assertThat(applied.getAppliedAt()).isEqualTo(Instant.parse("2026-02-14T18:30:00Z"));
        assertThat(applied.getAppliedTemperature()).isEqualTo(24.0);
        assertThat(applied.getAppliedVentilation()).isEqualTo(Ventilation.GOOD);
        assertThat(applied.getAppliedDryingClass()).isEqualTo(DryingClass.VERY_SLOW);

        // Les corrections du catalogue aussi.
        assertThat(catalog.search("Burnt Umber").stream()
                .filter(p -> p.getBrand().equals("Gamblin") && p.getName().equals("Burnt Umber"))
                .findFirst().orElseThrow().getTintHex()).isEqualTo("#C9B9AC");
    }
}
