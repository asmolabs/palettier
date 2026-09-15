package be.asmolabs.palettier.ui;

import java.awt.Taskbar;
import java.io.InputStream;
import java.util.ArrayList;
import java.util.List;
import javafx.scene.image.Image;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * L'icone de l'application, dans ses differentes tailles.
 *
 * <p>Le systeme choisit la taille qui lui convient : une icone dessinee pour 512 pixels
 * et reduite a 16 par le systeme devient une bouillie. Fournir chaque taille permet de
 * servir celle qui a ete rendue pour elle.</p>
 *
 * <p>Les fichiers sont produits par {@code AppIconGenerator}, cote tests.</p>
 */
final class AppIcons {

    private static final Logger log = LoggerFactory.getLogger(AppIcons.class);
    private static final int[] SIZES = {16, 32, 64, 128, 256, 512, 1024};
    private static final String PATTERN = "icon/palettier-%d.png";

    private AppIcons() {
    }

    /** Icones de la fenetre, de la plus petite a la plus grande. */
    static List<Image> all() {
        List<Image> images = new ArrayList<>();
        for (int size : SIZES) {
            try (InputStream stream = AppIcons.class.getResourceAsStream(PATTERN.formatted(size))) {
                if (stream != null) {
                    images.add(new Image(stream));
                }
            } catch (Exception e) {
                log.debug("Icone {} px illisible", size, e);
            }
        }
        return images;
    }

    /**
     * Icone du Dock ou de la barre des taches.
     *
     * <p>JavaFX ne s'en occupe pas : sur macOS, les icones de la fenetre ne remontent pas
     * jusqu'au Dock, il faut passer par l'API de bureau. Sans effet ailleurs, et sans
     * consequence si la plateforme ne le permet pas.</p>
     */
    static void applyToTaskbar() {
        try {
            if (!Taskbar.isTaskbarSupported()) {
                return;
            }
            Taskbar taskbar = Taskbar.getTaskbar();
            if (!taskbar.isSupported(Taskbar.Feature.ICON_IMAGE)) {
                return;
            }
            try (InputStream stream = AppIcons.class.getResourceAsStream(PATTERN.formatted(512))) {
                if (stream != null) {
                    taskbar.setIconImage(javax.imageio.ImageIO.read(stream));
                }
            }
        } catch (Exception e) {
            log.debug("Icone du Dock non appliquee", e);
        }
    }
}
