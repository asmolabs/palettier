package be.asmolabs.palettier.core.domain;

import jakarta.persistence.Column;
import jakarta.persistence.Entity;
import jakarta.persistence.EnumType;
import jakarta.persistence.Enumerated;
import jakarta.persistence.GeneratedValue;
import jakarta.persistence.GenerationType;
import jakarta.persistence.Id;
import jakarta.persistence.Table;
import jakarta.validation.constraints.DecimalMax;
import jakarta.validation.constraints.DecimalMin;
import jakarta.validation.constraints.NotNull;

/** Une etape de recette : une technique, un melange et un reglage de dilution. */
@Entity
@Table(name = "recipe_step")
public class RecipeStep {

    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;

    @NotNull
    @Enumerated(EnumType.STRING)
    @Column(nullable = false, length = 30)
    private Technique technique;

    /** Description libre du melange, par exemple "Terre d'ombre brulee + une pointe de noir". */
    @Column(name = "paint_mix", nullable = false)
    private String paintMix = "";

    @NotNull
    @Enumerated(EnumType.STRING)
    @Column(nullable = false, length = 30)
    private Medium medium = Medium.ODORLESS_THINNER;

    @DecimalMin("0.0")
    @DecimalMax("1.0")
    @Column(name = "medium_ratio", nullable = false)
    private double mediumRatio;

    @NotNull
    @Enumerated(EnumType.STRING)
    @Column(nullable = false, length = 20)
    private LayerThickness thickness = LayerThickness.THIN;

    @Column(length = 500)
    private String notes = "";

    protected RecipeStep() {
        // requis par JPA
    }

    public RecipeStep(Technique technique, String paintMix) {
        this(technique, paintMix, technique.defaultMedium(), technique.defaultRatio(), technique.typicalThickness());
    }

    public RecipeStep(Technique technique, String paintMix, Medium medium,
                      double mediumRatio, LayerThickness thickness) {
        this.technique = technique;
        this.paintMix = paintMix;
        this.medium = medium;
        this.mediumRatio = mediumRatio;
        this.thickness = thickness;
    }

    public Long getId() {
        return id;
    }

    public Technique getTechnique() {
        return technique;
    }

    public void setTechnique(Technique technique) {
        this.technique = technique;
    }

    public String getPaintMix() {
        return paintMix;
    }

    public void setPaintMix(String paintMix) {
        this.paintMix = paintMix;
    }

    public Medium getMedium() {
        return medium;
    }

    public void setMedium(Medium medium) {
        this.medium = medium;
    }

    public double getMediumRatio() {
        return mediumRatio;
    }

    public void setMediumRatio(double mediumRatio) {
        this.mediumRatio = mediumRatio;
    }

    public LayerThickness getThickness() {
        return thickness;
    }

    public void setThickness(LayerThickness thickness) {
        this.thickness = thickness;
    }

    public String getNotes() {
        return notes;
    }

    public void setNotes(String notes) {
        this.notes = notes;
    }

    @Override
    public String toString() {
        return technique.label() + " : " + paintMix;
    }
}
