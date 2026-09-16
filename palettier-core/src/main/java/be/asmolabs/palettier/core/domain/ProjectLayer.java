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
import java.time.Instant;

/**
 * Une couche du projet : le role tenu, la couleur visee, la technique.
 *
 * <p>Aucun dosage n'est stocke. La couleur visee est la decision ; le melange qui y
 * conduit est un calcul, refait a chaque ouverture avec la palette du moment.</p>
 *
 * <p>La pose, elle, est un fait et non une decision : une couche a ete peinte tel jour,
 * dans un atelier qui faisait telle temperature. C'est pourquoi les conditions sont
 * figees ici plutot que relues au moment de l'affichage -- l'huile a seche avec celles
 * du jour de la pose, pas avec celles d'aujourd'hui. Meme raison pour la vitesse de
 * sechage : elle vient des tubes reellement employes, et remanier la palette ensuite ne
 * doit pas reecrire le passe.</p>
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

    /** Date de pose, ou {@code null} tant que la couche n'est pas peinte. */
    @Column(name = "applied_at")
    private Instant appliedAt;

    /** Conditions de l'atelier au moment de la pose, figees avec elle. */
    @Column(name = "applied_temperature")
    private Double appliedTemperature;

    @Column(name = "applied_humidity")
    private Double appliedHumidity;

    @Enumerated(EnumType.STRING)
    @Column(name = "applied_ventilation", length = 10)
    private Ventilation appliedVentilation;

    /** Vitesse de sechage du melange effectivement pose. */
    @Enumerated(EnumType.STRING)
    @Column(name = "applied_drying_class", length = 10)
    private DryingClass appliedDryingClass;

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

    /** Vrai quand la couche a ete peinte et que son sechage court. */
    public boolean isApplied() {
        return appliedAt != null;
    }

    /**
     * Consigne la pose de la couche.
     *
     * <p>Les conditions sont prises en parametre plutot que lues ailleurs : ce sont
     * celles qui ont prevalu, et elles ne bougeront plus.</p>
     */
    public void markApplied(Instant when, double temperature, double humidity,
                            Ventilation ventilation, DryingClass dryingClass) {
        this.appliedAt = when;
        this.appliedTemperature = temperature;
        this.appliedHumidity = humidity;
        this.appliedVentilation = ventilation;
        this.appliedDryingClass = dryingClass;
    }

    /** Annule la pose : la couche redevient a peindre. Sert a corriger une fausse manoeuvre. */
    public void clearApplied() {
        this.appliedAt = null;
        this.appliedTemperature = null;
        this.appliedHumidity = null;
        this.appliedVentilation = null;
        this.appliedDryingClass = null;
    }

    public Instant getAppliedAt() {
        return appliedAt;
    }

    public Double getAppliedTemperature() {
        return appliedTemperature;
    }

    public Double getAppliedHumidity() {
        return appliedHumidity;
    }

    public Ventilation getAppliedVentilation() {
        return appliedVentilation;
    }

    public DryingClass getAppliedDryingClass() {
        return appliedDryingClass;
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
