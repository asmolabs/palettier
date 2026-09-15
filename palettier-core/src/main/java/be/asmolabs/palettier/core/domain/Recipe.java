package be.asmolabs.palettier.core.domain;

import jakarta.persistence.CascadeType;
import jakarta.persistence.Column;
import jakarta.persistence.Entity;
import jakarta.persistence.GeneratedValue;
import jakarta.persistence.GenerationType;
import jakarta.persistence.FetchType;
import jakarta.persistence.Id;
import jakarta.persistence.JoinColumn;
import jakarta.persistence.OneToMany;
import jakarta.persistence.OrderColumn;
import jakarta.persistence.Table;
import jakarta.validation.constraints.NotBlank;
import java.util.ArrayList;
import java.util.List;
import java.util.Objects;

/** Suite ordonnee d'etapes a l'huile pour un sujet donne. */
@Entity
@Table(name = "recipe")
public class Recipe {

    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;

    @NotBlank
    @Column(nullable = false)
    private String name;

    /** Ce sur quoi la recette s'applique : "casque allemand", "cape rouge", "peau 1/10"... */
    @Column(nullable = false)
    private String subject = "";

    @Column(length = 1000)
    private String notes = "";

    // chargement immediat : les etapes sont toujours affichees avec la recette,
    // et le client lourd n'a pas de session ouverte pendant le rendu
    @OneToMany(cascade = CascadeType.ALL, orphanRemoval = true, fetch = FetchType.EAGER)
    @JoinColumn(name = "recipe_id", nullable = false)
    @OrderColumn(name = "step_position")
    private List<RecipeStep> steps = new ArrayList<>();

    protected Recipe() {
        // requis par JPA
    }

    public Recipe(String name, String subject) {
        this.name = name;
        this.subject = subject;
    }

    public Recipe addStep(RecipeStep step) {
        steps.add(step);
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

    public String getNotes() {
        return notes;
    }

    public void setNotes(String notes) {
        this.notes = notes;
    }

    public List<RecipeStep> getSteps() {
        return steps;
    }

    public void setSteps(List<RecipeStep> steps) {
        this.steps.clear();
        this.steps.addAll(steps);
    }

    @Override
    public boolean equals(Object o) {
        return o instanceof Recipe other && id != null && Objects.equals(id, other.id);
    }

    @Override
    public int hashCode() {
        return Recipe.class.hashCode();
    }

    @Override
    public String toString() {
        return subject.isBlank() ? name : name + " (" + subject + ")";
    }
}
