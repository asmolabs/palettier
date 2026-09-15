package be.asmolabs.palettier.core.backup;

import be.asmolabs.palettier.core.color.Rgb;
import be.asmolabs.palettier.core.domain.DryingClass;
import be.asmolabs.palettier.core.domain.LayerThickness;
import be.asmolabs.palettier.core.domain.Medium;
import be.asmolabs.palettier.core.domain.OilPaint;
import be.asmolabs.palettier.core.domain.Opacity;
import be.asmolabs.palettier.core.domain.ProjectLayer;
import be.asmolabs.palettier.core.domain.ProjectZone;
import be.asmolabs.palettier.core.domain.Technique;
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
import java.util.LinkedHashSet;
import java.util.List;
import java.util.zip.ZipEntry;
import java.util.zip.ZipFile;
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
     * Ce qu'une restauration a fait, dans le detail.
     *
     * <p>Les comptes separent ce qui a ete ajoute de ce qui existait deja : une
     * restauration qui ne dit pas ce qu'elle a laisse de cote laisse un doute.</p>
     */
    public record ImportReport(int paintsUpdated, int paintsCreated,
                               int palettesAdded, int palettesSkipped,
                               int projectsAdded, int projectsSkipped,
                               int recipesAdded, int recipesSkipped,
                               int photos, List<String> warnings) {

        public String summary() {
            return ("%d tubes mis a jour, %d ajoutes  -  %d palettes, %d projets (%d photos), "
                    + "%d recettes restaures").formatted(paintsUpdated, paintsCreated,
                    palettesAdded, projectsAdded, photos, recipesAdded);
        }

        public int skipped() {
            return palettesSkipped + projectsSkipped + recipesSkipped;
        }
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

    // --- Restauration ------------------------------------------------------

    /**
     * Relit une archive et retablit ce qui manque.
     *
     * <p>Rien n'est ecrase. Les tubes du catalogue recoivent les corrections et
     * l'inventaire de l'archive, parce que ce sont des donnees de reference dont
     * l'archive porte une version plus a jour. Les palettes, projets et recettes qui
     * portent deja ce nom sont laisses tels quels et comptes a part : une restauration
     * ne doit jamais faire perdre un travail en cours.</p>
     *
     * @throws IOException              si l'archive est illisible
     * @throws IllegalArgumentException si ce n'est pas une sauvegarde Palettier
     */
    @Transactional
    public ImportReport importFrom(Path archive) throws IOException {
        try (ZipFile zip = new ZipFile(archive.toFile())) {
            ZipEntry document = zip.getEntry(DOCUMENT);
            if (document == null) {
                throw new IllegalArgumentException(
                        "Cette archive ne contient pas de " + DOCUMENT + " : ce n'est pas une sauvegarde Palettier.");
            }
            BackupModel.Backup backup = json.readValue(
                    zip.getInputStream(document).readAllBytes(), BackupModel.Backup.class);

            if (backup.formatVersion() > BackupModel.FORMAT_VERSION) {
                throw new IllegalArgumentException(
                        "Sauvegarde ecrite par une version plus recente (format %d, connu jusqu'a %d)."
                                .formatted(backup.formatVersion(), BackupModel.FORMAT_VERSION));
            }

            List<String> warnings = new ArrayList<>();
            int[] paintCounts = restorePaints(backup.paints());
            int[] paletteCounts = restorePalettes(backup.palettes(), warnings);
            int[] recipeCounts = restoreRecipes(backup.recipes());
            int[] projectCounts = restoreProjects(backup.projects(), zip, warnings);

            ImportReport report = new ImportReport(paintCounts[0], paintCounts[1],
                    paletteCounts[0], paletteCounts[1],
                    projectCounts[0], projectCounts[1],
                    recipeCounts[0], recipeCounts[1],
                    projectCounts[2], List.copyOf(warnings));
            log.info("Restauration depuis {} : {}", archive.getFileName(), report.summary());
            return report;
        }
    }

    /** @return nombre de tubes mis a jour, puis crees */
    private int[] restorePaints(List<BackupModel.Paint> saved) {
        int updated = 0;
        int created = 0;
        for (BackupModel.Paint entry : saved == null ? List.<BackupModel.Paint>of() : saved) {
            OilPaint paint = paints.findFirstByBrandIgnoreCaseAndNameIgnoreCase(
                    entry.brand(), entry.name()).orElse(null);
            if (paint == null) {
                paint = new OilPaint(entry.brand(), entry.name(), entry.code(), entry.hex(),
                        Opacity.valueOf(entry.opacity()), DryingClass.valueOf(entry.dryingClass()),
                        entry.tintingStrength(), new LinkedHashSet<>(entry.pigments()));
                created++;
            } else {
                paint.setHexColor(entry.hex());
                updated++;
            }
            paint.setTintHex(entry.tintHex());
            paint.setInStock(entry.owned());
            paint.setColorDerived(entry.colorDerived());
            paint.setNotes(entry.notes() == null ? "" : entry.notes());
            paints.save(paint);
        }
        return new int[]{updated, created};
    }

    /** @return nombre de palettes ajoutees, puis ignorees parce que deja presentes */
    private int[] restorePalettes(List<BackupModel.Palette> saved, List<String> warnings) {
        int added = 0;
        int skipped = 0;
        for (BackupModel.Palette entry : saved == null ? List.<BackupModel.Palette>of() : saved) {
            if (palettes.existsByNameIgnoreCase(entry.name())) {
                skipped++;
                continue;
            }
            be.asmolabs.palettier.core.domain.Palette palette =
                    new be.asmolabs.palettier.core.domain.Palette(entry.name(), entry.purpose());
            palette.setNotes(entry.notes() == null ? "" : entry.notes());
            entry.paints().forEach(ref -> resolve(ref, warnings).ifPresent(palette::add));
            palettes.save(palette);
            added++;
        }
        return new int[]{added, skipped};
    }

    /** @return nombre de recettes ajoutees, puis ignorees */
    private int[] restoreRecipes(List<BackupModel.Recipe> saved) {
        List<String> existing = recipes.findAllByOrderByNameAsc().stream()
                .map(be.asmolabs.palettier.core.domain.Recipe::getName)
                .toList();
        int added = 0;
        int skipped = 0;
        for (BackupModel.Recipe entry : saved == null ? List.<BackupModel.Recipe>of() : saved) {
            if (existing.stream().anyMatch(name -> name.equalsIgnoreCase(entry.name()))) {
                skipped++;
                continue;
            }
            var recipe = new be.asmolabs.palettier.core.domain.Recipe(entry.name(), entry.subject());
            recipe.setNotes(entry.notes() == null ? "" : entry.notes());
            for (BackupModel.Step step : entry.steps()) {
                var restored = new be.asmolabs.palettier.core.domain.RecipeStep(
                        Technique.valueOf(step.technique()), step.paintMix(),
                        Medium.valueOf(step.medium()), step.mediumRatio(),
                        LayerThickness.valueOf(step.thickness()));
                restored.setNotes(step.note() == null ? "" : step.note());
                recipe.addStep(restored);
            }
            recipes.save(recipe);
            added++;
        }
        return new int[]{added, skipped};
    }

    /** @return nombre de projets ajoutes, ignores, puis photos restaurees */
    private int[] restoreProjects(List<BackupModel.Project> saved, ZipFile zip, List<String> warnings)
            throws IOException {
        int added = 0;
        int skipped = 0;
        int photos = 0;

        for (BackupModel.Project entry : saved == null ? List.<BackupModel.Project>of() : saved) {
            if (projects.existsByNameIgnoreCase(entry.name())) {
                skipped++;
                continue;
            }
            var palette = entry.paletteName() == null ? null
                    : palettes.findAllByOrderByNameAsc().stream()
                        .filter(p -> p.getName().equalsIgnoreCase(entry.paletteName()))
                        .findFirst().orElse(null);
            if (palette == null && entry.paletteName() != null) {
                warnings.add("Projet \"%s\" : palette \"%s\" absente, les tubes conserves avec le projet suffisent."
                        .formatted(entry.name(), entry.paletteName()));
            }

            Project project = new Project(entry.name(), entry.subject(), palette);
            project.setApproach(entry.approach() == null ? "" : entry.approach());
            project.setNotes(entry.notes() == null ? "" : entry.notes());
            if (entry.createdAt() != null) {
                project.restoreCreatedAt(entry.createdAt());
            }

            List<OilPaint> frozen = new ArrayList<>();
            entry.paints().forEach(ref -> resolve(ref, warnings).ifPresent(frozen::add));
            project.setPaints(frozen);

            for (BackupModel.Zone zone : entry.zones()) {
                ProjectZone restored = new ProjectZone(zone.name(), zone.material(), zone.note());
                for (BackupModel.Layer layer : zone.layers()) {
                    restored.addLayer(new ProjectLayer(layer.role(), Rgb.ofHex(layer.targetHex()),
                            layer.technique(), layer.note(),
                            ProjectLayer.Kind.valueOf(layer.kind())));
                }
                project.addZone(restored);
            }

            for (BackupModel.Photo photo : entry.photos()) {
                byte[] data = readImage(zip, photo.file(), warnings);
                if (data != null) {
                    project.addPhoto(new ProjectPhoto(data,
                            ProjectPhoto.Role.valueOf(photo.role()), photo.caption()));
                    photos++;
                }
            }
            projects.save(project);
            added++;
        }
        return new int[]{added, skipped, photos};
    }

    /**
     * Lit une image de l'archive.
     *
     * <p>Le chemin annonce par le document est verifie avant lecture : une archive peut
     * venir de n'importe ou, et un nom d'entree n'est pas une source de confiance.</p>
     */
    private static byte[] readImage(ZipFile zip, String file, List<String> warnings) throws IOException {
        if (file == null || !file.startsWith(IMAGES + "/") || file.contains("..")) {
            warnings.add("Chemin d'image refuse : " + file);
            return null;
        }
        ZipEntry entry = zip.getEntry(file);
        if (entry == null) {
            warnings.add("Image annoncee mais absente de l'archive : " + file);
            return null;
        }
        return zip.getInputStream(entry).readAllBytes();
    }

    private java.util.Optional<OilPaint> resolve(BackupModel.PaintRef reference, List<String> warnings) {
        var found = paints.findFirstByBrandIgnoreCaseAndNameIgnoreCase(
                reference.brand(), reference.name());
        if (found.isEmpty()) {
            warnings.add("Tube introuvable : %s - %s".formatted(reference.brand(), reference.name()));
        }
        return found;
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
