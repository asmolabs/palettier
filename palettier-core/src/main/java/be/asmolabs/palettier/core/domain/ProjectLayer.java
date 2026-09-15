package be.asmolabs.palettier.core.domain;

import be.asmolabs.palettier.core.color.Rgb;
import jakarta.persistence.Column;
import jakarta.persistence.Entity;
import jakarta.persistence.EnumType;
import jakarta.persistence.Enumerated;
import jakarta.persistence.GeneratedValue;
import jakarta.persistence.GenerationType;
import jakarta.persistence.Id;
import jakarta.persistence.Table;
import jakarta.validation.constraints.Pattern;

/**
 * Une couche du projet : le role tenu, la couleur visee, la technique.
 *
 * <p>Aucun dosage n'est stocke. La couleur visee est la decision ; le melange qui y
 * conduit est un calcul, refait a chaque ouverture avec la palette du moment.</p>
 */
@Entity
@Table(name = "project_layer")
public class ProjectLayer {

    /**
     * A quoi sert la couche.
     *
     * <p>Le nom ne suffit pas a le dire : une variation locale s'appelle "rougeur des
     * pommettes", ce qu'aucune convention de nommage ne permet de reconnaitre. La
     * distinction est donc portee explicitement.</p>
     */
    public enum Kind {
        /** Une marche de l'echelle des valeurs : ombre, base ou lumiere. */
        LADDER,
        /** Une couleur locale, a peu pres a la valeur de la base. */
        ACCENT
    }

    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;

    /** "Ombre 2", "Base", "Lumiere 1"... */
    @Column(nullable = false, length = 40)
    private String role;

    @Pattern(regexp = "#[0-9A-Fa-f]{6}")
    @Column(name = "target_hex", nullable = false, length = 7)
    private String targetHex;

    @Column(length = 60)
    private String technique = "";

    @Column(length = 1000)
    private String note = "";

    @Enumerated(EnumType.STRING)
    @Column(nullable = false, length = 10)
    private Kind kind = Kind.LADDER;

    protected ProjectLayer() {
        // requis par JPA
    }

    public ProjectLayer(String role, Rgb target, String technique, String note) {
        this(role, target, technique, note, Kind.LADDER);
    }

    public ProjectLayer(String role, Rgb target, String technique, String note, Kind kind) {
        this.kind = kind;
        this.role = role;
        this.targetHex = target.toHex();
        this.technique = technique == null ? "" : technique;
        this.note = note == null ? "" : note;
    }

    public Rgb target() {
        return Rgb.ofHex(targetHex);
    }

    public Long getId() {
        return id;
    }

    public String getRole() {
        return role;
    }

    public Kind getKind() {
        return kind;
    }

    public void setKind(Kind kind) {
        this.kind = kind;
    }

    public String getTargetHex() {
        return targetHex;
    }

    public void setTargetHex(String targetHex) {
        this.targetHex = targetHex.toUpperCase();
    }

    public String getTechnique() {
        return technique;
    }

    public void setTechnique(String technique) {
        this.technique = technique;
    }

    public String getNote() {
        return note;
    }

    public void setNote(String note) {
        this.note = note;
    }
}
