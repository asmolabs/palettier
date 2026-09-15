package be.asmolabs.palettier.core.domain;

import jakarta.persistence.Column;
import jakarta.persistence.Entity;
import jakarta.persistence.FetchType;
import jakarta.persistence.GeneratedValue;
import jakarta.persistence.GenerationType;
import jakarta.persistence.Id;
import jakarta.persistence.JoinColumn;
import jakarta.persistence.JoinTable;
import jakarta.persistence.ManyToMany;
import jakarta.persistence.OrderColumn;
import jakarta.persistence.Table;
import jakarta.validation.constraints.NotBlank;
import java.util.ArrayList;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Objects;
import java.util.Set;

/**
 * Une selection nommee de tubes : la palette qu'on prepare pour un sujet donne.
 *
 * <p>Sur une figurine, on ne travaille jamais avec le catalogue entier mais avec six a
 * douze tubes choisis ensemble. La palette sert donc autant a s'organiser qu'a
 * restreindre les recherches : chercher un melange "dans ma palette" donne une reponse
 * utilisable, la meme recherche sur quatre cents tubes donne une reponse theorique.</p>
 */
@Entity
@Table(name = "palette")
public class Palette {

    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;

    @NotBlank
    @Column(nullable = false)
    private String name;

    /** Ce a quoi la palette est destinee : "carnations 1/10", "blindage vert olive"... */
    @Column(nullable = false)
    private String purpose = "";

    @Column(length = 1000)
    private String notes = "";

    @ManyToMany(fetch = FetchType.EAGER)
    @JoinTable(name = "palette_paint",
            joinColumns = @JoinColumn(name = "palette_id"),
            inverseJoinColumns = @JoinColumn(name = "paint_id"))
    @OrderColumn(name = "position")
    private List<OilPaint> paints = new ArrayList<>();

    protected Palette() {
        // requis par JPA
    }

    public Palette(String name, String purpose) {
        this.name = name;
        this.purpose = purpose;
    }

    /** Ajoute un tube s'il n'y est pas deja, et signale si la palette a change. */
    public boolean add(OilPaint paint) {
        if (paints.contains(paint)) {
            return false;
        }
        return paints.add(paint);
    }

    public boolean remove(OilPaint paint) {
        return paints.remove(paint);
    }

    /**
     * Classe de sechage imposee par la palette : celle de son tube le plus lent.
     * C'est elle qui commande le planning d'une seance menee avec ces couleurs.
     */
    public DryingClass slowestDryingClass() {
        return paints.stream()
                .map(OilPaint::getDryingClass)
                .reduce(DryingClass.FAST, DryingClass::slowest);
    }

    /** Pigments distincts presents dans la palette : au-dela de six, les melanges grisent. */
    public Set<String> pigments() {
        Set<String> codes = new LinkedHashSet<>();
        paints.forEach(paint -> codes.addAll(paint.getPigments()));
        return codes;
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

    public String getPurpose() {
        return purpose;
    }

    public void setPurpose(String purpose) {
        this.purpose = purpose;
    }

    public String getNotes() {
        return notes;
    }

    public void setNotes(String notes) {
        this.notes = notes;
    }

    public List<OilPaint> getPaints() {
        return paints;
    }

    public void setPaints(List<OilPaint> paints) {
        this.paints.clear();
        this.paints.addAll(paints);
    }

    @Override
    public boolean equals(Object o) {
        return o instanceof Palette other && id != null && Objects.equals(id, other.id);
    }

    @Override
    public int hashCode() {
        return Palette.class.hashCode();
    }

    @Override
    public String toString() {
        return purpose.isBlank() ? name : name + " (" + purpose + ")";
    }
}
