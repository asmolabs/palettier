package be.asmolabs.palettier.core.backup;

import static org.assertj.core.api.Assertions.assertThat;

import be.asmolabs.palettier.core.color.Rgb;
import be.asmolabs.palettier.core.domain.DryingClass;
import be.asmolabs.palettier.core.domain.Palette;
import be.asmolabs.palettier.core.domain.Project;
import be.asmolabs.palettier.core.domain.ProjectPhoto;
import be.asmolabs.palettier.core.domain.Ventilation;
import be.asmolabs.palettier.core.plan.PaintingPlan;
import be.asmolabs.palettier.core.service.DryingModels.Workshop;
import be.asmolabs.palettier.core.service.PaintCatalogService;
import be.asmolabs.palettier.core.service.PaletteService;
import be.asmolabs.palettier.core.service.ProjectService;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import java.awt.image.BufferedImage;
import java.io.ByteArrayOutputStream;
import java.nio.charset.StandardCharsets;
import java.nio.file.Path;
import java.time.Instant;
import java.util.List;
import java.util.zip.ZipEntry;
import java.util.zip.ZipFile;
import javax.imageio.ImageIO;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.transaction.annotation.Transactional;

@SpringBootTest
@Transactional
class BackupServiceTest {

    @Autowired
    private BackupService backup;

    @Autowired
    private ProjectService projects;

    @Autowired
    private PaletteService palettes;

    @Autowired
    private PaintCatalogService catalog;

    private Palette zorn() {
        return palettes.findAll().stream()
                .filter(p -> p.getName().startsWith("Palette Zorn"))
                .findFirst().orElseThrow();
    }

    private static PaintingPlan.Layer layer(String role, String hex) {
        return new PaintingPlan.Layer(role, Rgb.ofHex(hex), "Glacis", "note", null, Rgb.ofHex(hex), 0);
    }

    private Project projectWithPhoto(String name) throws Exception {
        PaintingPlan.Zone zone = new PaintingPlan.Zone("Visage", "Peau", "Par glacis.",
                layer("Base", "#C98F72"),
                List.of(layer("Ombre 1", "#8A5F4A")),
                List.of(layer("Lumiere 1", "#E0B49A")),
                List.of(layer("Rougeur des pommettes", "#C97A62")));
        Project project = projects.save(
                new PaintingPlan("Buste", "Zorn", "Approche.", List.of(zone)), zorn(), name);
        projects.addPhoto(project, image(), ProjectPhoto.Role.PIECE, "piece.png");
        return project;
    }

    private static byte[] image() throws Exception {
        BufferedImage img = new BufferedImage(120, 90, BufferedImage.TYPE_INT_RGB);
        ByteArrayOutputStream out = new ByteArrayOutputStream();
        ImageIO.write(img, "png", out);
        return out.toByteArray();
    }

    @Test
    @DisplayName("l'archive contient un JSON lisible et les photos en fichiers ordinaires")
    void theArchiveHoldsJsonAndPlainImages(@TempDir Path directory) throws Exception {
        projectWithPhoto("Mon grognard");
        Path archive = directory.resolve("sauvegarde.zip");

        BackupService.Summary summary = backup.export(archive);

        try (ZipFile zip = new ZipFile(archive.toFile())) {
            ZipEntry document = zip.getEntry(BackupService.DOCUMENT);
            assertThat(document).as("le document principal").isNotNull();

            JsonNode root = new ObjectMapper().readTree(
                    new String(zip.getInputStream(document).readAllBytes(), StandardCharsets.UTF_8));
            assertThat(root.path("formatVersion").asInt()).isEqualTo(BackupModel.FORMAT_VERSION);
            assertThat(root.path("projects")).isNotEmpty();

            // La photo est rangee a cote, pas encodee dans le JSON.
            String file = root.path("projects").get(0).path("photos").get(0).path("file").asText();
            assertThat(file).startsWith(BackupService.IMAGES + "/").endsWith(".jpg");
            assertThat(zip.getEntry(file)).as("le fichier image annonce").isNotNull();
            assertThat(zip.getInputStream(zip.getEntry(file)).readAllBytes()).isNotEmpty();
        }
        assertThat(summary.photos()).isEqualTo(1);
        assertThat(summary.paints()).isGreaterThan(400);
    }

    @Test
    @DisplayName("les tubes sont designes par marque et nom, jamais par un numero de base")
    void paintsAreReferencedByTheirNaturalKey(@TempDir Path directory) throws Exception {
        projectWithPhoto("Reference");
        Path archive = directory.resolve("sauvegarde.zip");
        backup.export(archive);

        JsonNode root = read(archive);
        JsonNode reference = root.path("palettes").get(0).path("paints").get(0);
        assertThat(reference.path("brand").asText()).isNotBlank();
        assertThat(reference.path("name").asText()).isNotBlank();
        assertThat(reference.has("id")).isFalse();
    }

    @Test
    @DisplayName("l'inventaire et les corrections du catalogue sont conserves")
    void inventoryAndCorrectionsAreKept(@TempDir Path directory) throws Exception {
        catalog.declareNothingOwned();
        // Plusieurs gammes ont un "Burnt Umber" et d'autres un "Burnt Umber/Brown Wash" :
        // on vise le tube exact, sans dependre de l'ordre du catalogue.
        var umber = catalog.search("Burnt Umber").stream()
                .filter(paint -> paint.getName().equals("Burnt Umber"))
                .findFirst()
                .orElseThrow();
        catalog.setOwned(umber, true);
        catalog.recordTint(umber, Rgb.ofHex("#C9B9AC"));

        Path archive = directory.resolve("sauvegarde.zip");
        backup.export(archive);

        JsonNode paints = read(archive).path("paints");
        JsonNode saved = null;
        for (JsonNode paint : paints) {
            if (paint.path("name").asText().equals("Burnt Umber") && paint.path("owned").asBoolean()) {
                saved = paint;
            }
        }
        assertThat(saved).as("le tube declare possede").isNotNull();
        assertThat(saved.path("tintHex").asText()).isEqualTo("#C9B9AC");
    }

    @Test
    @DisplayName("les variations locales gardent leur nature dans la sauvegarde")
    void accentsKeepTheirKind(@TempDir Path directory) throws Exception {
        projectWithPhoto("Avec variation");
        Path archive = directory.resolve("sauvegarde.zip");
        backup.export(archive);

        JsonNode layers = read(archive).path("projects").get(0).path("zones").get(0).path("layers");
        assertThat(layers).anySatisfy(layer -> {
            assertThat(layer.path("role").asText()).isEqualTo("Rougeur des pommettes");
            assertThat(layer.path("kind").asText()).isEqualTo("ACCENT");
        });
    }

    private static JsonNode read(Path archive) throws Exception {
        try (ZipFile zip = new ZipFile(archive.toFile())) {
            return new ObjectMapper().readTree(
                    new String(zip.getInputStream(zip.getEntry(BackupService.DOCUMENT)).readAllBytes(),
                            StandardCharsets.UTF_8));
        }
    }

    @Test
    @DisplayName("aller-retour : ce qui sort de l'archive est ce qui y etait entre")
    void aroundTrip(@TempDir Path directory) throws Exception {
        catalog.declareNothingOwned();
        // Plusieurs gammes ont un "Burnt Umber" et d'autres un "Burnt Umber/Brown Wash" :
        // on vise le tube exact, sans dependre de l'ordre du catalogue.
        var umber = catalog.search("Burnt Umber").stream()
                .filter(paint -> paint.getName().equals("Burnt Umber"))
                .findFirst()
                .orElseThrow();
        catalog.setOwned(umber, true);
        catalog.recordTint(umber, Rgb.ofHex("#C9B9AC"));
        Project original = projectWithPhoto("Grognard a restaurer");
        String paletteName = original.getPalette().getName();

        Path archive = directory.resolve("sauvegarde.zip");
        backup.export(archive);

        // On efface le travail, on garde le catalogue : la situation d'une machine neuve.
        projects.delete(original);
        catalog.declareNothingOwned();
        catalog.recordTint(umber, Rgb.ofHex("#FFFFFF"));

        BackupService.ImportReport report = backup.importFrom(archive);

        assertThat(report.projectsAdded()).isEqualTo(1);
        assertThat(report.photos()).isEqualTo(1);
        assertThat(report.paintsUpdated()).isGreaterThan(400);

        Project restored = projects.findAll().stream()
                .filter(p -> p.getName().equals("Grognard a restaurer"))
                .findFirst().orElseThrow();
        assertThat(restored.getPalette().getName()).isEqualTo(paletteName);
        assertThat(restored.getPhotos()).hasSize(1);
        assertThat(restored.getPaints()).isNotEmpty();
        assertThat(restored.getZones().getFirst().getLayers())
                .extracting(l -> l.getRole())
                .contains("Base", "Rougeur des pommettes");
        assertThat(restored.getZones().getFirst().getLayers())
                .filteredOn(l -> l.getKind() == be.asmolabs.palettier.core.domain.ProjectLayer.Kind.ACCENT)
                .hasSize(1);

        // Les corrections du catalogue reviennent aussi.
        assertThat(catalog.search("Burnt Umber").stream()
                .filter(paint -> paint.getName().equals("Burnt Umber"))
                .findFirst().orElseThrow().getTintHex()).isEqualTo("#C9B9AC");
        assertThat(catalog.countOwned()).isEqualTo(1);
    }

    @Test
    @DisplayName("l'avancement d'une piece survit a l'aller-retour")
    void appliedCoatsSurviveTheRoundTrip(@TempDir Path directory) throws Exception {
        Instant posed = Instant.parse("2026-02-14T18:30:00Z");
        Project original = projectWithPhoto("Piece a moitie peinte");
        projects.markApplied(original, 0, 0, new Workshop(24, 40, Ventilation.GOOD),
                DryingClass.VERY_SLOW, posed);

        Path archive = directory.resolve("sauvegarde.zip");
        backup.export(archive);
        projects.delete(original);

        backup.importFrom(archive);

        var layers = projects.findAll().stream()
                .filter(p -> p.getName().equals("Piece a moitie peinte"))
                .findFirst().orElseThrow()
                .getZones().getFirst().getLayers();

        // Une sauvegarde qui perdrait la pose rendrait la piece a peindre : ce qu'elle a
        // de plus precieux sur un travail en cours, c'est justement ou il en est.
        assertThat(layers.getFirst().getAppliedAt()).isEqualTo(posed);
        assertThat(layers.getFirst().getAppliedTemperature()).isEqualTo(24);
        assertThat(layers.getFirst().getAppliedVentilation()).isEqualTo(Ventilation.GOOD);
        assertThat(layers.getFirst().getAppliedDryingClass()).isEqualTo(DryingClass.VERY_SLOW);
        assertThat(layers).filteredOn(layer -> !layer.isApplied()).isNotEmpty();
    }

    @Test
    @DisplayName("une restauration n'ecrase jamais un travail en cours")
    void importNeverOverwritesExistingWork(@TempDir Path directory) throws Exception {
        Project original = projectWithPhoto("Projet garde");
        original.setNotes("Note ecrite apres la sauvegarde.");
        projects.save(original);

        Path archive = directory.resolve("sauvegarde.zip");
        backup.export(archive);
        original.setNotes("Note modifiee depuis.");
        projects.save(original);

        BackupService.ImportReport report = backup.importFrom(archive);

        assertThat(report.projectsAdded()).isZero();
        assertThat(report.projectsSkipped()).isEqualTo(1);
        assertThat(projects.findAll().stream()
                .filter(p -> p.getName().equals("Projet garde"))
                .findFirst().orElseThrow().getNotes())
                .as("le travail en cours l'emporte sur l'archive")
                .isEqualTo("Note modifiee depuis.");
    }

    @Test
    @DisplayName("reimporter la meme archive ne duplique rien")
    void importingTwiceChangesNothing(@TempDir Path directory) throws Exception {
        projectWithPhoto("Idempotent");
        Path archive = directory.resolve("sauvegarde.zip");
        backup.export(archive);

        backup.importFrom(archive);
        int after = projects.findAll().size();
        BackupService.ImportReport second = backup.importFrom(archive);

        assertThat(projects.findAll()).hasSize(after);
        assertThat(second.projectsAdded()).isZero();
    }

    @Test
    @DisplayName("une archive qui n'en est pas une est refusee avec un message clair")
    void anUnrelatedArchiveIsRefused(@TempDir Path directory) throws Exception {
        Path bogus = directory.resolve("autre.zip");
        try (var zip = new java.util.zip.ZipOutputStream(java.nio.file.Files.newOutputStream(bogus))) {
            zip.putNextEntry(new java.util.zip.ZipEntry("lisezmoi.txt"));
            zip.write("rien a voir".getBytes(StandardCharsets.UTF_8));
            zip.closeEntry();
        }

        org.assertj.core.api.Assertions.assertThatIllegalArgumentException()
                .isThrownBy(() -> backup.importFrom(bogus))
                .withMessageContaining("pas une sauvegarde Palettier");
    }
}
