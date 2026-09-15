package be.asmolabs.palettier.core.service;

import static org.assertj.core.api.Assertions.assertThat;

import be.asmolabs.palettier.core.color.Rgb;
import be.asmolabs.palettier.core.domain.OilPaint;
import be.asmolabs.palettier.core.domain.Palette;
import be.asmolabs.palettier.core.domain.Project;
import be.asmolabs.palettier.core.plan.PaintingPlan;
import java.util.List;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.transaction.annotation.Transactional;

@SpringBootTest
@Transactional
class ProjectServiceTest {

    @Autowired
    private ProjectService projects;

    @Autowired
    private PaletteService palettes;

    @Autowired
    private PaintCatalogService catalog;

    private static PaintingPlan.Layer layer(String role, String hex) {
        return new PaintingPlan.Layer(role, Rgb.ofHex(hex), "Glacis", "note", null, Rgb.ofHex(hex), 0);
    }

    private static PaintingPlan plan() {
        PaintingPlan.Zone zone = new PaintingPlan.Zone("Visage", "Peau", "Par glacis.",
                layer("Base", "#C98F72"),
                List.of(layer("Ombre 1", "#8A5F4A"), layer("Ombre 2", "#5A3B2E")),
                List.of(layer("Lumiere 1", "#E0B49A"), layer("Lumiere 2", "#F2D8C4")));
        return new PaintingPlan("Buste de test", "Palette Zorn", "Palette courte.", List.of(zone));
    }

    private Palette zorn() {
        return palettes.findAll().stream()
                .filter(p -> p.getName().startsWith("Palette Zorn"))
                .findFirst()
                .orElseThrow();
    }

    @Test
    @DisplayName("un plan enregistre conserve ses zones, ses roles et ses couleurs visees")
    void savedPlanKeepsItsDecisions() {
        Project project = projects.save(plan(), zorn(), "Mon grognard");

        assertThat(project.getName()).isEqualTo("Mon grognard");
        assertThat(project.getZones()).hasSize(1);
        assertThat(project.getZones().getFirst().getLayers())
                .extracting(l -> l.getRole())
                .containsExactly("Ombre 2", "Ombre 1", "Base", "Lumiere 1", "Lumiere 2");
        assertThat(project.getZones().getFirst().getLayers())
                .extracting(l -> l.getTargetHex())
                .contains("#C98F72", "#5A3B2E", "#F2D8C4");
    }

    @Test
    @DisplayName("les dosages sont recalcules a l'ouverture, ils ne sont pas stockes")
    void recipesAreRecomputedOnRead() {
        Project project = projects.save(plan(), zorn(), "Recalcul");

        PaintingPlan restored = projects.plan(project);

        assertThat(restored.zones()).hasSize(1);
        assertThat(restored.zones().getFirst().layers())
                .allSatisfy(l -> assertThat(l.recipe())
                        .as("chaque couche doit porter une recette calculee")
                        .isNotNull());
        assertThat(restored.zones().getFirst().layers())
                .extracting(PaintingPlan.Layer::role)
                .containsExactly("Ombre 2", "Ombre 1", "Base", "Lumiere 1", "Lumiere 2");
    }

    @Test
    @DisplayName("enrichir la palette ne touche pas a un projet deja enregistre")
    void enrichingThePaletteLeavesExistingProjectsAlone() {
        Palette palette = palettes.create("Palette pauvre", "test");
        palettes.addPaint(palette, catalog.search("Titanium White").getFirst());
        Project project = projects.save(plan(), palette, "Fige");

        double before = projects.plan(project).zones().getFirst().base().deltaE();
        palettes.addPaint(palette, catalog.search("Burnt Umber").getFirst());
        palettes.addPaint(palette, catalog.search("Yellow Ochre").getFirst());

        assertThat(projects.plan(project).zones().getFirst().base().deltaE())
                .as("un plan enregistre ne change pas dans le dos de celui qui l'a etabli")
                .isEqualTo(before);
        assertThat(project.divergesFromPalette()).isTrue();
    }

    @Test
    @DisplayName("resynchroniser sur la palette fait profiter le projet des nouveaux tubes")
    void syncingWithThePaletteImprovesTheRecipes() {
        Palette palette = palettes.create("Palette a enrichir", "test");
        palettes.addPaint(palette, catalog.search("Titanium White").getFirst());
        Project project = projects.save(plan(), palette, "A resynchroniser");
        double before = projects.plan(project).zones().getFirst().base().deltaE();

        palettes.addPaint(palette, catalog.search("Burnt Umber").getFirst());
        palettes.addPaint(palette, catalog.search("Yellow Ochre").getFirst());
        projects.syncWithPalette(project);

        assertThat(projects.plan(project).zones().getFirst().base().deltaE())
                .as("les memes couleurs visees, mieux approchees : %.1f puis apres reprise", before)
                .isLessThan(before);
        assertThat(project.divergesFromPalette()).isFalse();
    }

    @Test
    @DisplayName("les tubes sont figes avec le projet, pas seulement reference")
    void paintsAreSnapshotted() {
        Project project = projects.save(plan(), zorn(), "Instantane");

        assertThat(project.getPaints()).hasSize(4).isEqualTo(zorn().getPaints());
    }

    @Test
    @DisplayName("deux projets ne peuvent pas porter le meme nom")
    void namesAreMadeUnique() {
        Project first = projects.save(plan(), zorn(), "Grognard");
        Project second = projects.save(plan(), zorn(), "Grognard");

        assertThat(second.getName()).isNotEqualTo(first.getName()).startsWith("Grognard");
    }

    @Test
    @DisplayName("un projet dont la palette a disparu reste lisible")
    void aProjectSurvivesItsPalette() {
        Palette temporary = palettes.create("Palette ephemere", "");
        palettes.addPaint(temporary, catalog.findAll().getFirst());
        Project project = projects.save(plan(), temporary, "Sans palette");
        project.setPalette(null);

        PaintingPlan restored = projects.plan(project);

        assertThat(restored.paletteName()).contains("conserves avec le projet");
        assertThat(restored.zones().getFirst().base().target().toHex()).isEqualTo("#C98F72");
        assertThat(restored.zones().getFirst().base().recipe())
                .as("les tubes figes survivent a la suppression de la palette")
                .isNotNull();
    }

    @Test
    @DisplayName("corriger une couche change la couleur visee, et la recette suit")
    void editingALayerChangesTheTargetAndTheRecipe() {
        Project project = projects.save(plan(), zorn(), "A corriger");
        String before = projects.plan(project).zones().getFirst().base().recipe().describe();

        projects.updateLayer(project, 0, 2, Rgb.ofHex("#3A2A20"), "Filtre", "Plus sombre.");

        PaintingPlan after = projects.plan(project);
        assertThat(after.zones().getFirst().base().target().toHex()).isEqualTo("#3A2A20");
        assertThat(after.zones().getFirst().base().technique()).isEqualTo("Filtre");
        assertThat(after.zones().getFirst().base().recipe().describe())
                .as("un autre but appelle un autre melange")
                .isNotEqualTo(before);
    }

    @Test
    @DisplayName("le rang d'une couche dans le plan correspond a son rang en base")
    void layerOrderMatchesStorage() {
        Project project = projects.save(plan(), zorn(), "Ordre");

        List<String> stored = project.getZones().getFirst().getLayers().stream()
                .map(l -> l.getRole()).toList();
        List<String> shown = projects.plan(project).zones().getFirst().layers().stream()
                .map(PaintingPlan.Layer::role).toList();

        // C'est ce rang qui relie un clic dans l'interface a la ligne a modifier.
        assertThat(shown).isEqualTo(stored);
    }

    @Test
    @DisplayName("une photo ajoutee est reduite avant d'etre rangee")
    void photosAreReducedBeforeStorage() throws Exception {
        Project project = projects.save(plan(), zorn(), "Avec photo");
        byte[] large = bigImage(2400, 1800);

        projects.addPhoto(project, large, be.asmolabs.palettier.core.domain.ProjectPhoto.Role.PIECE, "photo.png");

        assertThat(project.getPhotos()).hasSize(1);
        byte[] stored = project.getPhotos().getFirst().getData();
        assertThat(stored.length).isLessThan(large.length);

        java.awt.image.BufferedImage read = javax.imageio.ImageIO.read(
                new java.io.ByteArrayInputStream(stored));
        assertThat(Math.max(read.getWidth(), read.getHeight())).isLessThanOrEqualTo(1024);
    }

    @Test
    @DisplayName("les photos accompagnent le plan des l'enregistrement")
    void photosAreKeptWithThePlan() throws Exception {
        var photo = new be.asmolabs.palettier.core.domain.ProjectPhoto(
                be.asmolabs.palettier.core.image.Photos.prepare(bigImage(800, 600)),
                be.asmolabs.palettier.core.domain.ProjectPhoto.Role.PIECE, "piece.png");

        Project project = projects.save(plan(), zorn(), "Photos jointes", List.of(photo));

        assertThat(project.getPhotos()).hasSize(1);
        assertThat(project.getPhotos().getFirst().getCaption()).isEqualTo("piece.png");
    }

    private static byte[] bigImage(int width, int height) throws Exception {
        java.awt.image.BufferedImage image =
                new java.awt.image.BufferedImage(width, height, java.awt.image.BufferedImage.TYPE_INT_RGB);
        java.awt.Graphics2D g = image.createGraphics();
        g.setPaint(new java.awt.GradientPaint(0, 0, java.awt.Color.ORANGE, width, height, java.awt.Color.BLUE));
        g.fillRect(0, 0, width, height);
        g.dispose();
        java.io.ByteArrayOutputStream out = new java.io.ByteArrayOutputStream();
        javax.imageio.ImageIO.write(image, "png", out);
        return out.toByteArray();
    }
}
