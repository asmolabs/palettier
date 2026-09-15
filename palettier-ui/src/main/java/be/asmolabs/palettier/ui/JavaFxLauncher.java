package be.asmolabs.palettier.ui;

import javafx.application.Application;
import javafx.application.Platform;
import javafx.stage.Stage;
import org.springframework.boot.builder.SpringApplicationBuilder;
import org.springframework.context.ConfigurableApplicationContext;

/**
 * Fait le pont entre le cycle de vie JavaFX et celui de Spring.
 *
 * <p>Le contexte est construit dans {@code init()}, hors du fil d'affichage, puis la
 * fenetre principale est notifiee par un evenement Spring plutot que par un appel
 * direct : les vues restent ainsi de simples beans, sans connaitre le lanceur.</p>
 */
public class JavaFxLauncher extends Application {

    private ConfigurableApplicationContext context;

    @Override
    public void init() {
        context = new SpringApplicationBuilder(PalettierApplication.class)
                .web(org.springframework.boot.WebApplicationType.NONE)
                .run(getParameters().getRaw().toArray(String[]::new));
    }

    @Override
    public void start(Stage stage) {
        context.publishEvent(new StageReadyEvent(stage));
    }

    @Override
    public void stop() {
        if (context != null) {
            context.close();
        }
        Platform.exit();
    }
}
