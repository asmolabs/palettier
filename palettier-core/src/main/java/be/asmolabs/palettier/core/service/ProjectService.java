package be.asmolabs.palettier.core.service;

import be.asmolabs.palettier.core.color.Colors;
import be.asmolabs.palettier.core.color.Rgb;
import be.asmolabs.palettier.core.domain.Palette;
import be.asmolabs.palettier.core.domain.Project;
import be.asmolabs.palettier.core.domain.ProjectLayer;
import be.asmolabs.palettier.core.domain.ProjectPhoto;
import be.asmolabs.palettier.core.domain.ProjectZone;
import be.asmolabs.palettier.core.plan.PaintingPlan;
import be.asmolabs.palettier.core.image.Photos;
import be.asmolabs.palettier.core.repository.ProjectRepository;
import be.asmolabs.palettier.core.service.MixModels.MixSuggestion;
import java.util.ArrayList;
import java.util.List;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

/**
 * Conserve les plans de peinture sous forme de projets, et les restitue.
 *
 * <p>Un projet garde les decisions -- quelles zones, quelle couleur viser, quelle
 * technique -- et la liste des tubes avec lesquels elles ont ete prises. Les dosages,
 * eux, sont recalcules a chaque lecture a partir de ces deux elements.</p>
 *
 * <p>Figer les tubes est ce qui rend un plan reproductible : remanier ou supprimer une
 * palette ne touche pas aux projets deja etablis. Profiter d'une palette enrichie reste
 * possible, mais demande un geste : {@link #syncWithPalette(Project)}. Un plan enregistre
 * ne doit pas changer dans le dos de celui qui l'a etabli.</p>
 */
@Service
@Transactional(readOnly = true)
public class ProjectService {

    /** Nombre maximal de tubes par melange lors de la restitution. */
    private static final int MAX_PAINTS = 3;

    private final ProjectRepository repository;
    private final ColorMixService mixer;

    public ProjectService(ProjectRepository repository, ColorMixService mixer) {
        this.repository = repository;
        this.mixer = mixer;
    }

    public List<Project> findAll() {
        return repository.findAllByOrderByCreatedAtDesc();
    }

    /** Enregistre un plan sous un nom unique et renvoie le projet cree. */
    @Transactional
    public Project save(PaintingPlan plan, Palette palette, String wantedName) {
        return save(plan, palette, wantedName, List.of());
    }

    /**
     * @param photos images a conserver avec le projet, reduites au passage
     */
    @Transactional
    public Project save(PaintingPlan plan, Palette palette, String wantedName,
                        List<ProjectPhoto> photos) {
        Project project = new Project(uniqueName(wantedName), plan.subject(), palette);
        project.setApproach(plan.approach());
        // Instantane des tubes : c'est avec eux que le plan a ete etabli.
        project.setPaints(palette == null ? List.of() : List.copyOf(palette.getPaints()));

        for (PaintingPlan.Zone zone : plan.zones()) {
            ProjectZone stored = new ProjectZone(zone.name(), zone.material(), zone.note());
            for (PaintingPlan.Layer layer : zone.layers()) {
                stored.addLayer(new ProjectLayer(layer.role(), layer.target(),
                        layer.technique(), layer.note(), ProjectLayer.Kind.LADDER));
            }
            for (PaintingPlan.Layer accent : zone.accents()) {
                stored.addLayer(new ProjectLayer(accent.role(), accent.target(),
                        accent.technique(), accent.note(), ProjectLayer.Kind.ACCENT));
            }
            project.addZone(stored);
        }
        photos.forEach(project::addPhoto);
        return repository.save(project);
    }

    /**
     * Attache une photo au projet.
     *
     * <p>L'image est reduite ici et non chez l'appelant : une base qui accueille des
     * photos doit garantir elle-meme qu'elles restent d'une taille raisonnable.</p>
     */
    @Transactional
    public Project addPhoto(Project project, byte[] original, ProjectPhoto.Role role, String caption) {
        project.addPhoto(new ProjectPhoto(Photos.prepare(original), role, caption));
        return repository.save(project);
    }

    @Transactional
    public Project removePhoto(Project project, ProjectPhoto photo) {
        project.getPhotos().remove(photo);
        return repository.save(project);
    }

    /**
     * Modifie une couche : la couleur visee, la technique, la note.
     *
     * <p>C'est la correction du peintre sur la proposition recue. Seule la decision est
     * enregistree ; le dosage qui en decoule est recalcule a la lecture suivante.</p>
     */
    @Transactional
    public Project updateLayer(Project project, int zoneIndex, int layerIndex,
                               Rgb target, String technique, String note) {
        ProjectLayer layer = project.getZones().get(zoneIndex).getLayers().get(layerIndex);
        layer.setTargetHex(target.toHex());
        layer.setTechnique(technique);
        layer.setNote(note);
        return repository.save(project);
    }

    /** Modifie l'intitule d'une zone. */
    @Transactional
    public Project updateZone(Project project, int zoneIndex, String name, String material, String note) {
        ProjectZone zone = project.getZones().get(zoneIndex);
        zone.setName(name);
        zone.setMaterial(material);
        zone.setNote(note);
        return repository.save(project);
    }

    @Transactional
    public Project save(Project project) {
        return repository.save(project);
    }

    /**
     * Reprend la composition actuelle de la palette d'origine.
     *
     * <p>A employer quand la palette s'est enrichie et qu'on veut en faire profiter un
     * projet ancien : les couleurs visees ne bougent pas, seuls les melanges qui y menent
     * sont recalcules avec les tubes du jour.</p>
     */
    @Transactional
    public Project syncWithPalette(Project project) {
        if (project.getPalette() != null) {
            project.setPaints(List.copyOf(project.getPalette().getPaints()));
        }
        return repository.save(project);
    }

    @Transactional
    public Project rename(Project project, String name) {
        project.setName(uniqueName(name));
        return repository.save(project);
    }

    @Transactional
    public void delete(Project project) {
        repository.delete(project);
    }

    /**
     * Reconstitue le plan d'un projet, dosages recalcules avec ses propres tubes.
     *
     * <p>La palette peut avoir ete remaniee ou supprimee depuis : cela ne change rien,
     * le projet porte sa propre liste.</p>
     */
    public PaintingPlan plan(Project project) {
        List<PaintingPlan.Zone> zones = new ArrayList<>();
        List<be.asmolabs.palettier.core.domain.OilPaint> paints = project.effectivePaints();

        for (ProjectZone zone : project.getZones()) {
            List<PaintingPlan.Layer> ladder = zone.getLayers().stream()
                    .filter(layer -> layer.getKind() == ProjectLayer.Kind.LADDER)
                    .map(layer -> recompute(layer, paints))
                    .toList();
            List<PaintingPlan.Layer> accents = zone.getLayers().stream()
                    .filter(layer -> layer.getKind() == ProjectLayer.Kind.ACCENT)
                    .map(layer -> recompute(layer, paints))
                    .toList();
            zones.add(rebuild(zone, ladder, accents));
        }

        String paletteName = project.getPalette() == null
                ? "%d tubes conserves avec le projet".formatted(paints.size())
                : project.getPalette().getName();
        return new PaintingPlan(project.getSubject(), paletteName, project.getApproach(), List.copyOf(zones));
    }

    private PaintingPlan.Layer recompute(ProjectLayer layer,
                                         List<be.asmolabs.palettier.core.domain.OilPaint> paints) {
        Rgb target = layer.target();
        if (paints.isEmpty()) {
            return new PaintingPlan.Layer(layer.getRole(), target, layer.getTechnique(),
                    layer.getNote(), null, target, 0);
        }
        List<MixSuggestion> found = mixer.suggestMixes(target, paints, 1, MAX_PAINTS);
        if (found.isEmpty()) {
            return new PaintingPlan.Layer(layer.getRole(), target, layer.getTechnique(),
                    layer.getNote(), null, target, 0);
        }
        MixSuggestion best = found.getFirst();
        return new PaintingPlan.Layer(layer.getRole(), target, layer.getTechnique(), layer.getNote(),
                best, best.color(), Colors.deltaE2000(target, best.color()));
    }

    /**
     * Remet les couches dans la forme attendue par le plan. Les roles ont ete conserves
     * tels quels : ce sont eux qui disent ou va chaque couche.
     */
    private static PaintingPlan.Zone rebuild(ProjectZone zone, List<PaintingPlan.Layer> layers,
                                             List<PaintingPlan.Layer> accents) {
        List<PaintingPlan.Layer> shadows = layers.stream()
                .filter(layer -> layer.role().toLowerCase().startsWith("ombre"))
                .toList();
        List<PaintingPlan.Layer> highlights = layers.stream()
                .filter(layer -> layer.role().toLowerCase().startsWith("lumiere"))
                .toList();
        PaintingPlan.Layer base = layers.stream()
                .filter(layer -> layer.role().equalsIgnoreCase("base"))
                .findFirst()
                .orElse(layers.isEmpty() ? null : layers.get(layers.size() / 2));

        // Les ombres ont ete enregistrees de la plus sombre a la plus claire : on rend
        // l'ordre attendu, de la plus legere a la plus profonde.
        return new PaintingPlan.Zone(zone.getName(), zone.getMaterial(), zone.getNote(),
                base, shadows.reversed(), highlights, accents);
    }

    private String uniqueName(String wanted) {
        String candidate = wanted == null || wanted.isBlank() ? "Projet" : wanted.trim();
        String base = candidate;
        int suffix = 2;
        while (repository.existsByNameIgnoreCase(candidate)) {
            candidate = base + " " + suffix++;
        }
        return candidate;
    }
}
