package be.asmolabs.palettier.core.service;

import static org.assertj.core.api.Assertions.assertThat;

import be.asmolabs.palettier.core.color.Rgb;
import be.asmolabs.palettier.core.domain.DryingClass;
import be.asmolabs.palettier.core.domain.Project;
import be.asmolabs.palettier.core.domain.ProjectPhoto;
import be.asmolabs.palettier.core.plan.PaintingPlan;
import be.asmolabs.palettier.core.service.DryingModels.Workshop;
import jakarta.persistence.EntityManager;
import jakarta.persistence.PersistenceContext;
import java.awt.image.BufferedImage;
import java.io.ByteArrayOutputStream;
import java.time.Instant;
import java.util.List;
import javax.imageio.ImageIO;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.transaction.annotation.Transactional;

/**
 * Les photos sont chargees a la demande, et cela ne doit rien casser.
 *
 * <p>Le risque est reel : un projet lu sans ses photos puis reenregistre pourrait les
 * effacer, la liaison etant en {@code orphanRemoval}. Le contexte de persistance est
 * vide entre chaque etape pour que ces tests voient ce que voit l'application, et non
 * des entites encore chaudes.</p>
 */
@SpringBootTest
@Transactional
class ProjectPhotoLoadingTest {

    @Autowired
    private ProjectService projects;

    @PersistenceContext
    private EntityManager entityManager;

    private static PaintingPlan.Layer layer(String role, String hex) {
        return new PaintingPlan.Layer(role, Rgb.ofHex(hex), "Glacis", "", null, Rgb.ofHex(hex), 0);
    }

    private Project pieceWithPhoto(String name) throws Exception {
        PaintingPlan.Zone zone = new PaintingPlan.Zone("Visage", "Peau", "",
                layer("Base", "#C98F72"),
                List.of(layer("Ombre 1", "#8A5F4A")),
                List.of(layer("Lumiere 1", "#E0B49A")));
        Project project = projects.save(new PaintingPlan("Buste", "", "", List.of(zone)), null, name);
        return projects.addPhoto(project, image(), ProjectPhoto.Role.PIECE, "piece.png");
    }

    private static byte[] image() throws Exception {
        BufferedImage img = new BufferedImage(200, 150, BufferedImage.TYPE_INT_RGB);
        ByteArrayOutputStream out = new ByteArrayOutputStream();
        ImageIO.write(img, "png", out);
        return out.toByteArray();
    }

    /** Vide le contexte : sans cela, tout serait deja charge et les tests ne prouveraient rien. */
    private void detachEverything() {
        entityManager.flush();
        entityManager.clear();
    }

    @Test
    @DisplayName("la liste des projets se lit sans les photos, qui se demandent")
    void photosAreFetchedOnDemand() throws Exception {
        Project saved = pieceWithPhoto("Avec photo");
        detachEverything();

        Project fromList = projects.findAll().stream()
                .filter(p -> p.getId().equals(saved.getId()))
                .findFirst().orElseThrow();

        assertThat(projects.withPhotos(fromList).getPhotos())
                .as("photos demandees explicitement")
                .hasSize(1);
    }

    @Test
    @DisplayName("modifier un projet lu sans ses photos ne les efface pas")
    void savingAProjectReadWithoutPhotosKeepsThem() throws Exception {
        Project saved = pieceWithPhoto("Couche a marquer");
        detachEverything();

        // Exactement le chemin de l'interface : on prend le projet dans la liste, donc
        // sans ses photos, et on coche une couche.
        Project fromList = projects.findAll().stream()
                .filter(p -> p.getId().equals(saved.getId()))
                .findFirst().orElseThrow();
        Project updated = projects.markApplied(fromList, 0, 0, Workshop.standard(),
                DryingClass.FAST, Instant.parse("2026-03-01T10:00:00Z"));
        detachEverything();

        assertThat(projects.withPhotos(updated).getPhotos())
                .as("photos apres une modification de couche")
                .hasSize(1);
    }

    @Test
    @DisplayName("ajouter une photo a un projet lu sans elles conserve les precedentes")
    void addingAPhotoKeepsTheOnesAlreadyThere() throws Exception {
        Project saved = pieceWithPhoto("Deux photos");
        detachEverything();

        Project fromList = projects.findAll().stream()
                .filter(p -> p.getId().equals(saved.getId()))
                .findFirst().orElseThrow();
        Project updated = projects.addPhoto(fromList, image(), ProjectPhoto.Role.REFERENCE, "reference.png");

        assertThat(updated.getPhotos()).extracting(ProjectPhoto::getCaption)
                .containsExactly("piece.png", "reference.png");
    }

    @Test
    @DisplayName("retirer une photo se fait sur son identifiant, pas sur l'instance")
    void removingAPhotoWorksAcrossReloads() throws Exception {
        Project saved = pieceWithPhoto("Photo a retirer");
        ProjectPhoto photo = projects.withPhotos(saved).getPhotos().getFirst();
        detachEverything();

        Project fromList = projects.findAll().stream()
                .filter(p -> p.getId().equals(saved.getId()))
                .findFirst().orElseThrow();
        Project updated = projects.removePhoto(fromList, photo);

        assertThat(updated.getPhotos()).isEmpty();
    }
}
