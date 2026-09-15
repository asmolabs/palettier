package be.asmolabs.palettier.core.domain;

import jakarta.persistence.CascadeType;
import jakarta.persistence.Column;
import jakarta.persistence.Entity;
import jakarta.persistence.FetchType;
import jakarta.persistence.GeneratedValue;
import jakarta.persistence.GenerationType;
import jakarta.persistence.Id;
import jakarta.persistence.JoinColumn;
import jakarta.persistence.JoinTable;
import jakarta.persistence.ManyToMany;
import jakarta.persistence.ManyToOne;
import jakarta.persistence.OneToMany;
import jakarta.persistence.OrderColumn;
import jakarta.persistence.Table;
import jakarta.validation.constraints.NotBlank;
import java.time.Instant;
import java.util.ArrayList;
import java.util.List;
import java.util.Objects;

/**
 * Une piece en cours, avec le plan de peinture retenu pour elle.
 *
 * <p>Ce qui est conserve, ce sont les <em>decisions</em> : le decoupage en zones, la
 * couleur visee pour chaque couche, la technique. Les dosages n'y sont pas -- ils se
 * recalculent -- mais les tubes avec lesquels ils se calculent, si.</p>
 *
 * <p>Le projet garde en effet sa propre liste de tubes, figee au moment de
 * l'enregistrement, et non un simple renvoi vers la palette. C'est ce qui fait qu'un
 * plan reste reproductible : retirer un tube d'une palette, la remanier ou la supprimer
 * ne change rien aux projets deja etablis. La palette reste liee pour memoire, et un
 * geste explicite permet de resynchroniser le projet dessus quand on le souhaite.</p>
 */
@Entity
@Table(name = "project")
public class Project {

    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;

    @NotBlank
    @Column(nullable = false)
    private String name;

    /** Description du sujet, telle qu'elle a servi a etablir le plan. */
    @Column(length = 1000)
    private String subject = "";

    /** Parti pris general retenu pour la piece. */
    @Column(length = 2000)
    private String approach = "";

    @Column(length = 2000)
    private String notes = "";

    /** La palette d'origine, gardee pour memoire et pour pouvoir resynchroniser. */
    @ManyToOne(fetch = FetchType.EAGER)
    @JoinColumn(name = "palette_id")
    private Palette palette;

    /**
     * Les tubes reellement disponibles quand le plan a ete etabli.
     *
     * <p>C'est cette liste qui sert aux calculs, pas la palette : un projet doit pouvoir
     * etre repris a l'identique des mois plus tard, quoi qu'il soit arrive a la palette
     * entre-temps.</p>
     */
    @ManyToMany(fetch = FetchType.EAGER)
    @JoinTable(name = "project_paint",
            joinColumns = @JoinColumn(name = "project_id"),
            inverseJoinColumns = @JoinColumn(name = "paint_id"))
    @OrderColumn(name = "paint_position")
    private List<OilPaint> paints = new ArrayList<>();

    @Column(name = "created_at", nullable = false)
    private Instant createdAt = Instant.now();

    @OneToMany(cascade = CascadeType.ALL, orphanRemoval = true, fetch = FetchType.EAGER)
    @JoinColumn(name = "project_id", nullable = false)
    @OrderColumn(name = "zone_position")
    private List<ProjectZone> zones = new ArrayList<>();

    /** Photos de la piece : celle qui a servi au plan, les references, l'avancement. */
    @OneToMany(cascade = CascadeType.ALL, orphanRemoval = true, fetch = FetchType.EAGER)
    @JoinColumn(name = "project_id", nullable = false)
    @OrderColumn(name = "photo_position")
    private List<ProjectPhoto> photos = new ArrayList<>();

    protected Project() {
        // requis par JPA
    }

    public Project(String name, String subject, Palette palette) {
        this.name = name;
        this.subject = subject == null ? "" : subject;
        this.palette = palette;
    }

    public Project addZone(ProjectZone zone) {
        zones.add(zone);
        return this;
    }

    public Long getId() {
        return id;
    }

    public String getName() {
        return name;
    }

    public void setName(String name) {
        this.name = name;
    }

    public String getSubject() {
        return subject;
    }

    public void setSubject(String subject) {
        this.subject = subject;
    }

    public String getApproach() {
        return approach;
    }

    public void setApproach(String approach) {
        this.approach = approach;
    }

    public String getNotes() {
        return notes;
    }

    public void setNotes(String notes) {
        this.notes = notes;
    }

    public Palette getPalette() {
        return palette;
    }

    public void setPalette(Palette palette) {
        this.palette = palette;
    }

    public Instant getCreatedAt() {
        return createdAt;
    }

    public List<ProjectZone> getZones() {
        return zones;
    }

    public List<OilPaint> getPaints() {
        return paints;
    }

    public void setPaints(List<OilPaint> paints) {
        this.paints.clear();
        this.paints.addAll(paints);
    }

    /**
     * Les tubes a employer pour les calculs : ceux figes avec le projet, et a defaut ceux
     * de la palette. Le repli couvre les projets enregistres avant que l'instantane
     * existe.
     */
    public List<OilPaint> effectivePaints() {
        if (!paints.isEmpty()) {
            return paints;
        }
        return palette == null ? List.of() : palette.getPaints();
    }

    /**
     * Vrai si la palette a ete remaniee depuis l'enregistrement. L'ecart n'est pas une
     * erreur : c'est une information, a l'utilisateur de decider s'il veut en profiter.
     */
    public boolean divergesFromPalette() {
        if (palette == null || paints.isEmpty()) {
            return false;
        }
        return !paints.equals(palette.getPaints());
    }

    public List<ProjectPhoto> getPhotos() {
        return photos;
    }

    public Project addPhoto(ProjectPhoto photo) {
        photos.add(photo);
        return this;
    }

    @Override
    public boolean equals(Object o) {
        return o instanceof Project other && id != null && Objects.equals(id, other.id);
    }

    @Override
    public int hashCode() {
        return Project.class.hashCode();
    }

    @Override
    public String toString() {
        return name;
    }
}
