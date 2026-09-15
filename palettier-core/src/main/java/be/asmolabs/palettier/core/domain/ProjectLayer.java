package be.asmolabs.palettier.core.domain;

import be.asmolabs.palettier.core.color.Rgb;
import jakarta.persistence.Column;
import jakarta.persistence.Entity;
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

    protected ProjectLayer() {
        // requis par JPA
    }

    public ProjectLayer(String role, Rgb target, String technique, String note) {
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
