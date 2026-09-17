package be.asmolabs.palettier.core.service;

import static org.assertj.core.api.Assertions.assertThat;

import be.asmolabs.palettier.core.color.Rgb;
import be.asmolabs.palettier.core.domain.Project;
import be.asmolabs.palettier.core.domain.ProjectLayer;
import be.asmolabs.palettier.core.domain.ProjectZone;
import be.asmolabs.palettier.core.service.ProgressCheckService.Observed;
import java.awt.Color;
import java.awt.Graphics2D;
import java.awt.image.BufferedImage;
import java.io.ByteArrayOutputStream;
import java.util.List;
import javax.imageio.ImageIO;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;

class ProgressCheckServiceTest {

    private final ProgressCheckService service = new ProgressCheckService();

    private static Project piece() {
        Project project = new Project("Grognard", "Buste", null);
        ProjectZone zone = new ProjectZone("Visage", "Peau", "");
        zone.addLayer(new ProjectLayer("Base", Rgb.ofHex("#C98F72"), "Glacis", "", ProjectLayer.Kind.LADDER));
        zone.addLayer(new ProjectLayer("Ombre 1", Rgb.ofHex("#8A5F4A"), "Glacis", "", ProjectLayer.Kind.LADDER));
        project.addZone(zone);
        return project;
    }

    /** Une image de deux aplats, dont l'un est exactement une couleur visee. */
    private static byte[] twoTone(String left, String right) throws Exception {
        BufferedImage image = new BufferedImage(200, 100, BufferedImage.TYPE_INT_RGB);
        Graphics2D g = image.createGraphics();
        g.setColor(Color.decode(left));
        g.fillRect(0, 0, 100, 100);
        g.setColor(Color.decode(right));
        g.fillRect(100, 0, 100, 100);
        g.dispose();

        ByteArrayOutputStream out = new ByteArrayOutputStream();
        ImageIO.write(image, "png", out);
        return out.toByteArray();
    }

    @Test
    @DisplayName("une teinte posee comme prevu est reconnue, et l'ecart est nul")
    void apaintedTargetIsRecognised() throws Exception {
        List<Observed> observed = service.compare(piece(), twoTone("#C98F72", "#8A5F4A"), 4);

        assertThat(observed).isNotEmpty();
        assertThat(observed).anySatisfy(seen -> {
            assertThat(seen.role()).isEqualTo("Base");
            assertThat(seen.deltaE()).isLessThan(2);
            assertThat(seen.verdict()).contains("vous y etes");
        });
        assertThat(observed).anySatisfy(seen -> assertThat(seen.role()).isEqualTo("Ombre 1"));
    }

    @Test
    @DisplayName("une teinte qui n'etait prevue nulle part est signalee comme telle")
    void anunplannedColourIsCalledOut() throws Exception {
        // Un vert franc : rien dans ce plan de carnation ne s'en approche.
        List<Observed> observed = service.compare(piece(), twoTone("#1FA337", "#1FA337"), 3);

        assertThat(observed).allSatisfy(seen -> {
            assertThat(seen.matchesPlan()).isFalse();
            assertThat(seen.verdict()).contains("Rien de prevu");
        });
    }

    @Test
    @DisplayName("un projet sans zone ne fait pas echouer la comparaison")
    void aprojectWithoutZonesIsTolerated() throws Exception {
        Project empty = new Project("Vide", "Rien", null);

        List<Observed> observed = service.compare(empty, twoTone("#C98F72", "#8A5F4A"), 3);

        assertThat(observed).isNotEmpty();
        assertThat(observed).allSatisfy(seen -> assertThat(seen.matchesPlan()).isFalse());
    }
}
