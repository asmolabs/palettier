package be.asmolabs.palettier.core.domain;

import be.asmolabs.palettier.core.color.Colorant;
import be.asmolabs.palettier.core.color.Rgb;
import jakarta.persistence.CollectionTable;
import jakarta.persistence.Column;
import jakarta.persistence.ElementCollection;
import jakarta.persistence.Entity;
import jakarta.persistence.EnumType;
import jakarta.persistence.Enumerated;
import jakarta.persistence.FetchType;
import jakarta.persistence.GeneratedValue;
import jakarta.persistence.GenerationType;
import jakarta.persistence.Id;
import jakarta.persistence.JoinColumn;
import jakarta.persistence.Table;
import jakarta.persistence.UniqueConstraint;
import jakarta.validation.constraints.DecimalMax;
import jakarta.validation.constraints.DecimalMin;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.NotNull;
import jakarta.validation.constraints.Pattern;
import java.util.LinkedHashSet;
import java.util.Objects;
import java.util.Set;

/** Un tube d'huile du catalogue ou de l'etagere du peintre. */
@Entity
// La cle naturelle est la marque et le nom : plusieurs gammes ne publient aucune
// reference, et une contrainte sur la reference ferait alors collisionner tous les
// tubes qui n'en ont pas.
@Table(name = "oil_paint",
        uniqueConstraints = @UniqueConstraint(name = "uk_paint_brand_name", columnNames = {"brand", "name"}))
public class OilPaint {

    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;

    @NotBlank
    @Column(nullable = false)
    private String brand;

    @NotBlank
    @Column(nullable = false)
    private String name;

    @Column(nullable = false)
    private String code = "";

    /** Codes normalises des pigments, par exemple PBr7 ou PW6. */
    @ElementCollection(fetch = FetchType.EAGER)
    @CollectionTable(name = "oil_paint_pigment", joinColumns = @JoinColumn(name = "paint_id"))
    @Column(name = "pigment", nullable = false)
    private Set<String> pigments = new LinkedHashSet<>();

    @NotBlank
    @Pattern(regexp = "#[0-9A-Fa-f]{6}", message = "La couleur doit etre au format #RRGGBB")
    @Column(name = "hex_color", nullable = false, length = 7)
    private String hexColor;

    @NotNull
    @Enumerated(EnumType.STRING)
    @Column(nullable = false, length = 20)
    private Opacity opacity = Opacity.SEMI_OPAQUE;

    @NotNull
    @Enumerated(EnumType.STRING)
    @Column(name = "drying_class", nullable = false, length = 20)
    private DryingClass dryingClass = DryingClass.MEDIUM;

    /**
     * Pouvoir colorant relatif, entre 0,05 et 1. Une teinte a fort pouvoir colorant
     * domine le melange bien avant d'etre majoritaire en volume.
     */
    @DecimalMin("0.05")
    @DecimalMax("1.0")
    @Column(name = "tinting_strength", nullable = false)
    private double tintingStrength = 0.5;

    /**
     * Couleur de l'huile coupee de blanc, si elle a ete relevee.
     *
     * <p>Sa presence fait basculer les melanges impliquant ce tube vers le modele de
     * Kubelka-Munk a deux constantes. Sans elle, le comportement est celui du modele a
     * constante unique, inchange.</p>
     */
    @Pattern(regexp = "|#[0-9A-Fa-f]{6}", message = "La teinte diluee doit etre au format #RRGGBB")
    @Column(name = "tint_hex", length = 7)
    private String tintHex;

    /**
     * Vrai quand la teinte n'a pas ete relevee mais deduite des pigments declares.
     * L'interface le signale : c'est une approximation, pas une mesure.
     */
    @Column(name = "color_derived", nullable = false)
    private boolean colorDerived;

    /**
     * Vrai quand les pigments viennent du fabricant, et non d'une reconstitution.
     *
     * <p>Distinction qui merite d'etre visible : ce sont les pigments qui donnent la
     * vitesse de sechage, donc le planning. Une fiche reconstituee de memoire peut se
     * tromper de pigment, et le peintre doit savoir sur quoi il s'appuie.</p>
     */
    @Column(name = "pigments_verified", nullable = false)
    private boolean pigmentsVerified;

    /**
     * Vrai pour un tube saisi par le peintre, et non livre avec l'application.
     *
     * <p>Distinction necessaire au menage : une fiche livree qui disparait des fichiers
     * de gamme doit etre retiree, une fiche saisie a la main ne doit jamais l'etre.</p>
     */
    @Column(name = "user_added", nullable = false)
    private boolean userAdded;

    /** Vrai si le tube est effectivement sur l'etagere du peintre. */
    @Column(name = "in_stock", nullable = false)
    private boolean inStock = true;

    @Column(length = 500)
    private String notes = "";

    protected OilPaint() {
        // requis par JPA
    }

    public OilPaint(String brand, String name, String code, String hexColor,
                    Opacity opacity, DryingClass dryingClass, double tintingStrength,
                    Set<String> pigments) {
        this.brand = brand;
        this.name = name;
        this.code = code == null ? "" : code;
        this.hexColor = hexColor.toUpperCase();
        this.opacity = opacity;
        this.dryingClass = dryingClass;
        this.tintingStrength = tintingStrength;
        this.pigments = new LinkedHashSet<>(pigments);
    }

    public Rgb color() {
        return Rgb.ofHex(hexColor);
    }

    /**
     * Constantes de melange du tube. A deux constantes si sa teinte diluee est connue,
     * a constante unique sinon.
     */
    public Colorant colorant() {
        return tintHex == null || tintHex.isBlank()
                ? Colorant.ofMasstone(color())
                : Colorant.ofMasstoneAndTint(color(), Rgb.ofHex(tintHex),
                        Colorant.REFERENCE_TINT_CONCENTRATION);
    }

    public String getTintHex() {
        return tintHex;
    }

    public void setTintHex(String tintHex) {
        this.tintHex = tintHex == null || tintHex.isBlank() ? null : tintHex.toUpperCase();
    }

    public boolean isColorDerived() {
        return colorDerived;
    }

    public void setColorDerived(boolean colorDerived) {
        this.colorDerived = colorDerived;
    }

    /** Libelle court pour les listes deroulantes : "Marque - Nom". */
    public String displayName() {
        return brand + " - " + name;
    }

    public Long getId() {
        return id;
    }

    public String getBrand() {
        return brand;
    }

    public void setBrand(String brand) {
        this.brand = brand;
    }

    public String getName() {
        return name;
    }

    public void setName(String name) {
        this.name = name;
    }

    public String getCode() {
        return code;
    }

    public void setCode(String code) {
        this.code = code;
    }

    public Set<String> getPigments() {
        return pigments;
    }

    public void setPigments(Set<String> pigments) {
        this.pigments = new LinkedHashSet<>(pigments);
    }

    public String getHexColor() {
        return hexColor;
    }

    public void setHexColor(String hexColor) {
        this.hexColor = hexColor.toUpperCase();
    }

    public Opacity getOpacity() {
        return opacity;
    }

    public void setOpacity(Opacity opacity) {
        this.opacity = opacity;
    }

    public DryingClass getDryingClass() {
        return dryingClass;
    }

    public void setDryingClass(DryingClass dryingClass) {
        this.dryingClass = dryingClass;
    }

    public double getTintingStrength() {
        return tintingStrength;
    }

    public void setTintingStrength(double tintingStrength) {
        this.tintingStrength = tintingStrength;
    }

    public boolean isPigmentsVerified() {
        return pigmentsVerified;
    }

    public void setPigmentsVerified(boolean pigmentsVerified) {
        this.pigmentsVerified = pigmentsVerified;
    }

    public boolean isUserAdded() {
        return userAdded;
    }

    public void setUserAdded(boolean userAdded) {
        this.userAdded = userAdded;
    }

    public boolean isInStock() {
        return inStock;
    }

    public void setInStock(boolean inStock) {
        this.inStock = inStock;
    }

    public String getNotes() {
        return notes;
    }

    public void setNotes(String notes) {
        this.notes = notes;
    }

    @Override
    public boolean equals(Object o) {
        return o instanceof OilPaint other && id != null && Objects.equals(id, other.id);
    }

    @Override
    public int hashCode() {
        return OilPaint.class.hashCode();
    }

    @Override
    public String toString() {
        return displayName();
    }
}
