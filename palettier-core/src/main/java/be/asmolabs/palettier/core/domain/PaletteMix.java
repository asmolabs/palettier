package be.asmolabs.palettier.core.domain;

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
 * Un melange pose sur la palette, avec l'heure a laquelle il l'a ete.
 *
 * <p>Le reste de l'application traite les melanges comme des calculs : on les refait a
 * la demande, ils n'ont pas d'existence propre. Celui-la en a une. A l'huile, un melange
 * reste travaillable des heures et survit parfois plusieurs jours sur une palette humide
 * -- c'est un fait physique, pose sur une vraie palette, et personne ne se souvient au
 * matin de ce qu'il a melange la veille ni de quand.</p>
 *
 * <p>Les conditions sont figees a la preparation, comme pour une couche posee : c'est
 * l'atelier de ce moment-la qui commande le temps ouvert.</p>
 */
@Entity
@Table(name = "palette_mix")
public class PaletteMix {

    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;

    /** Ce a quoi il sert : "gris rompu des ombres", "carnation de base". */
    @Column(nullable = false, length = 120)
    private String name;

    @Pattern(regexp = "#[0-9A-Fa-f]{6}")
    @Column(name = "hex_color", nullable = false, length = 7)
    private String hexColor;

    /** De quoi il est fait, en clair : c'est ce qui permet de le refaire. */
    @Column(length = 500)
    private String recipe = "";

    /** Vitesse du tube le plus lent du melange : c'est elle qui tient le temps ouvert. */
    @Enumerated(EnumType.STRING)
    @Column(name = "drying_class", nullable = false, length = 10)
    private DryingClass dryingClass = DryingClass.MEDIUM;

    @Column(name = "mixed_at", nullable = false)
    private Instant mixedAt = Instant.now();

    @Column(name = "temperature", nullable = false)
    private double temperature = 20;

    @Column(name = "humidity", nullable = false)
    private double humidity = 50;

    @Enumerated(EnumType.STRING)
    @Column(nullable = false, length = 10)
    private Ventilation ventilation = Ventilation.NORMAL;

    protected PaletteMix() {
        // requis par JPA
    }

    public PaletteMix(String name, String hexColor, String recipe, DryingClass dryingClass,
                      Instant mixedAt, double temperature, double humidity, Ventilation ventilation) {
        this.name = name;
        this.hexColor = hexColor.toUpperCase();
        this.recipe = recipe == null ? "" : recipe;
        this.dryingClass = dryingClass;
        this.mixedAt = mixedAt;
        this.temperature = temperature;
        this.humidity = humidity;
        this.ventilation = ventilation;
    }

    public Long getId() {
        return id;
    }

    public String getName() {
        return name;
    }

    public String getHexColor() {
        return hexColor;
    }

    public String getRecipe() {
        return recipe;
    }

    public DryingClass getDryingClass() {
        return dryingClass;
    }

    public Instant getMixedAt() {
        return mixedAt;
    }

    public double getTemperature() {
        return temperature;
    }

    public double getHumidity() {
        return humidity;
    }

    public Ventilation getVentilation() {
        return ventilation;
    }
}
