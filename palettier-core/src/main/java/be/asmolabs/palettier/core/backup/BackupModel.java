package be.asmolabs.palettier.core.backup;

import java.time.Instant;
import java.util.List;

/**
 * Forme de la sauvegarde, telle qu'elle est ecrite dans le fichier JSON.
 *
 * <p>Volontairement independante des entites : une sauvegarde doit rester lisible et
 * reimportable meme apres un remaniement du modele. Les tubes y sont designes par leur
 * marque et leur nom, jamais par un identifiant de base -- un numero de ligne ne veut
 * rien dire dans une autre installation.</p>
 */
public final class BackupModel {

    /** Version du format, pour qu'une relecture ulterieure sache a quoi elle a affaire. */
    public static final int FORMAT_VERSION = 1;

    private BackupModel() {
    }

    public record Backup(int formatVersion,
                         Instant exportedAt,
                         String application,
                         List<Paint> paints,
                         List<Palette> palettes,
                         List<Project> projects,
                         List<Recipe> recipes) {
    }

    /**
     * Un tube tel que le peintre l'a chez lui : ses corrections comprises.
     *
     * @param tintHex teinte coupee de blanc, ou {@code null} si elle n'est pas connue
     * @param owned   vrai si le peintre declare le posseder
     */
    public record Paint(String brand, String name, String code, List<String> pigments,
                        String hex, String tintHex, String opacity, String dryingClass,
                        double tintingStrength, boolean owned, boolean colorDerived, String notes) {
    }

    /** Une palette, ses tubes designes par marque et nom. */
    public record Palette(String name, String purpose, String notes, List<PaintRef> paints) {
    }

    /** Renvoi vers un tube, par sa cle naturelle. */
    public record PaintRef(String brand, String name) {
    }

    public record Project(String name, String subject, String approach, String notes,
                          String paletteName, Instant createdAt,
                          List<PaintRef> paints, List<Zone> zones, List<Photo> photos) {
    }

    public record Zone(String name, String material, String note, List<Layer> layers) {
    }

    /** @param kind LADDER pour une marche du degrade, ACCENT pour une variation locale */
    public record Layer(String role, String targetHex, String technique, String note, String kind) {
    }

    /**
     * Une photo, rangee a cote du JSON plutot qu'encodee dedans.
     *
     * @param file chemin du fichier dans l'archive, par exemple "images/projet-1-photo-2.jpg"
     */
    public record Photo(String file, String role, String caption, Instant addedAt) {
    }

    public record Recipe(String name, String subject, String notes, List<Step> steps) {
    }

    public record Step(String technique, String paintMix, String medium,
                       double mediumRatio, String thickness, String note) {
    }
}
