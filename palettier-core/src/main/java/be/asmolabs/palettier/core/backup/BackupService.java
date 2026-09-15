package be.asmolabs.palettier.core.backup;

import be.asmolabs.palettier.core.domain.OilPaint;
import be.asmolabs.palettier.core.domain.Project;
import be.asmolabs.palettier.core.domain.ProjectPhoto;
import be.asmolabs.palettier.core.repository.OilPaintRepository;
import be.asmolabs.palettier.core.repository.PaletteRepository;
import be.asmolabs.palettier.core.repository.ProjectRepository;
import be.asmolabs.palettier.core.repository.RecipeRepository;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.SerializationFeature;
import com.fasterxml.jackson.datatype.jsr310.JavaTimeModule;
import java.io.IOException;
import java.io.OutputStream;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Instant;
import java.util.ArrayList;
import java.util.List;
import java.util.zip.ZipEntry;
import java.util.zip.ZipOutputStream;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

/**
 * Sauvegarde complete de ce qui appartient au peintre.
 *
 * <p>Une archive contenant un fichier JSON lisible et les photos a cote, en fichiers
 * ordinaires. Ni base de donnees a restaurer, ni images noyees en base64 : on doit
 * pouvoir ouvrir l'archive, lire le JSON dans un editeur et regarder les photos dans
 * une visionneuse, des annees plus tard et sans l'application.</p>
 *
 * <p>Ce qui est sauvegarde, c'est le travail : les palettes, les projets, les recettes,
 * l'inventaire et les corrections apportees au catalogue. Le catalogue livre d'origine,
 * lui, se retrouve dans n'importe quelle installation.</p>
 */
@Service
public class BackupService {

    private static final Logger log = LoggerFactory.getLogger(BackupService.class);

    /** Nom du document principal dans l'archive. */
    public static final String DOCUMENT = "palettier.json";

    /** Dossier des photos dans l'archive. */
    public static final String IMAGES = "images";

    private final OilPaintRepository paints;
    private final PaletteRepository palettes;
    private final ProjectRepository projects;
    private final RecipeRepository recipes;
    private final ObjectMapper json;

    public BackupService(OilPaintRepository paints, PaletteRepository palettes,
                         ProjectRepository projects, RecipeRepository recipes) {
        this.paints = paints;
        this.palettes = palettes;
        this.projects = projects;
        this.recipes = recipes;
        this.json = new ObjectMapper()
                .registerModule(new JavaTimeModule())
                .disable(SerializationFeature.WRITE_DATES_AS_TIMESTAMPS)
                .enable(SerializationFeature.INDENT_OUTPUT);
    }

    /** Ce qu'une archive contient, pour le dire a l'utilisateur. */
    public record Summary(int paints, int palettes, int projects, int recipes, int photos, long bytes) {
    }

    /**
     * Ecrit l'archive.
     *
     * @param destination fichier zip a creer ou remplacer
     * @throws IOException si l'archive ne peut pas etre ecrite
     */
    @Transactional(readOnly = true)
    public Summary export(Path destination) throws IOException {
        List<BackupModel.Photo> photoIndex = new ArrayList<>();
        List<ProjectPhoto> photoData = new ArrayList<>();

        List<BackupModel.Project> exportedProjects = new ArrayList<>();
        for (Project project : projects.findAllByOrderByCreatedAtDesc()) {
            exportedProjects.add(toBackup(project, photoIndex, photoData));
        }

        BackupModel.Backup backup = new BackupModel.Backup(
                BackupModel.FORMAT_VERSION, Instant.now(), "Palettier",
                paints.findAllByOrderByBrandAscNameAsc().stream().map(BackupService::toBackup).toList(),
                palettes.findAllByOrderByNameAsc().stream().map(BackupService::toBackup).toList(),
                List.copyOf(exportedProjects),
                recipes.findAllByOrderByNameAsc().stream().map(BackupService::toBackup).toList());

        try (ZipOutputStream zip = new ZipOutputStream(Files.newOutputStream(destination))) {
            write(zip, DOCUMENT, json.writeValueAsBytes(backup));
            for (int i = 0; i < photoIndex.size(); i++) {
                write(zip, photoIndex.get(i).file(), photoData.get(i).getData());
            }
        }

        Summary summary = new Summary(backup.paints().size(), backup.palettes().size(),
                backup.projects().size(), backup.recipes().size(), photoIndex.size(),
                Files.size(destination));
        log.info("Sauvegarde ecrite : {} ({} projets, {} photos, {} ko)",
                destination, summary.projects(), summary.photos(), summary.bytes() / 1024);
        return summary;
    }

    private static void write(ZipOutputStream zip, String name, byte[] content) throws IOException {
        zip.putNextEntry(new ZipEntry(name));
        zip.write(content);
        zip.closeEntry();
    }

    // --- Conversions -------------------------------------------------------

    private static BackupModel.Paint toBackup(OilPaint paint) {
        return new BackupModel.Paint(paint.getBrand(), paint.getName(), paint.getCode(),
                List.copyOf(paint.getPigments()), paint.getHexColor(), paint.getTintHex(),
                paint.getOpacity().name(), paint.getDryingClass().name(),
                paint.getTintingStrength(), paint.isInStock(), paint.isColorDerived(),
                paint.getNotes());
    }

    private static BackupModel.Palette toBackup(be.asmolabs.palettier.core.domain.Palette palette) {
        return new BackupModel.Palette(palette.getName(), palette.getPurpose(), palette.getNotes(),
                palette.getPaints().stream().map(BackupService::reference).toList());
    }

    private static BackupModel.PaintRef reference(OilPaint paint) {
        return new BackupModel.PaintRef(paint.getBrand(), paint.getName());
    }

    private static BackupModel.Project toBackup(Project project,
                                                List<BackupModel.Photo> index,
                                                List<ProjectPhoto> data) {
        List<BackupModel.Photo> photos = new ArrayList<>();
        for (ProjectPhoto photo : project.getPhotos()) {
            // Un nom lisible et unique : on doit pouvoir ranger l'archive a la main.
            String file = "%s/%s-%02d.jpg".formatted(IMAGES, slug(project.getName()), photos.size() + 1);
            BackupModel.Photo entry = new BackupModel.Photo(file, photo.getRole().name(),
                    photo.getCaption(), photo.getAddedAt());
            photos.add(entry);
            index.add(entry);
            data.add(photo);
        }

        List<BackupModel.Zone> zones = project.getZones().stream()
                .map(zone -> new BackupModel.Zone(zone.getName(), zone.getMaterial(), zone.getNote(),
                        zone.getLayers().stream()
                                .map(layer -> new BackupModel.Layer(layer.getRole(), layer.getTargetHex(),
                                        layer.getTechnique(), layer.getNote(), layer.getKind().name()))
                                .toList()))
                .toList();

        return new BackupModel.Project(project.getName(), project.getSubject(), project.getApproach(),
                project.getNotes(),
                project.getPalette() == null ? null : project.getPalette().getName(),
                project.getCreatedAt(),
                project.getPaints().stream().map(BackupService::reference).toList(),
                zones, List.copyOf(photos));
    }

    private static BackupModel.Recipe toBackup(be.asmolabs.palettier.core.domain.Recipe recipe) {
        return new BackupModel.Recipe(recipe.getName(), recipe.getSubject(), recipe.getNotes(),
                recipe.getSteps().stream()
                        .map(step -> new BackupModel.Step(step.getTechnique().name(), step.getPaintMix(),
                                step.getMedium().name(), step.getMediumRatio(),
                                step.getThickness().name(), step.getNotes()))
                        .toList());
    }

    /** Nom de fichier sur, derive du nom du projet. */
    private static String slug(String name) {
        String plain = java.text.Normalizer.normalize(name, java.text.Normalizer.Form.NFD)
                .replaceAll("\\p{M}", "")
                .toLowerCase()
                .replaceAll("[^a-z0-9]+", "-")
                .replaceAll("(^-|-$)", "");
        return plain.isBlank() ? "projet" : plain.substring(0, Math.min(40, plain.length()));
    }
}
