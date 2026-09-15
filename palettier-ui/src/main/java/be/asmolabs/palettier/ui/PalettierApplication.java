package be.asmolabs.palettier.ui;

import be.asmolabs.palettier.ai.AiConfiguration;
import be.asmolabs.palettier.core.config.CoreConfiguration;
import javafx.application.Application;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.context.annotation.Import;

/**
 * Point d'entree du logiciel.
 *
 * <p>Cette classe ne fait que passer la main a JavaFX : c'est {@link JavaFxLauncher}
 * qui demarre le contexte Spring, de facon que le cycle de vie de l'application
 * graphique et celui du contexte restent alignes.</p>
 */
@SpringBootApplication
@Import({CoreConfiguration.class, AiConfiguration.class})
public class PalettierApplication {

    public static void main(String[] args) {
        Application.launch(JavaFxLauncher.class, args);
    }
}
