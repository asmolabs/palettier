package be.asmolabs.palettier.ui;

import static org.assertj.core.api.Assertions.assertThat;

import be.asmolabs.palettier.ai.AiConfiguration;
import be.asmolabs.palettier.core.config.CoreConfiguration;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.TimeUnit;
import javafx.application.Platform;
import javafx.scene.Node;
import javafx.scene.Scene;
import javafx.scene.layout.StackPane;
import org.junit.jupiter.api.Assumptions;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.context.annotation.Import;
import org.springframework.test.context.TestPropertySource;

/**
 * Construit chaque section de l'application.
 *
 * <p>Les sections ne sont bâties qu'a leur premiere ouverture : c'est bon pour le
 * demarrage, mais cela veut dire qu'une vue cassee ne se manifeste qu'au moment ou l'on
 * clique dessus. Ce test les construit toutes, avec le vrai contexte Spring et la vraie
 * feuille de style, pour que la casse se voie ici plutot que chez le peintre.</p>
 */
@SpringBootTest
@TestPropertySource(properties = {
        "spring.ai.model.chat=none",
        "spring.jpa.hibernate.ddl-auto=validate",
        "spring.datasource.url=jdbc:h2:mem:views-smoke;DB_CLOSE_DELAY=-1"
})
class AllViewsSmokeTest {

    /**
     * Le toolkit demarre avant le contexte Spring, et non dans le test.
     *
     * <p>Les vues creent leurs controles des l'initialisation de leurs champs : construire
     * le bean revient donc a construire des noeuds JavaFX, ce qui exige un toolkit deja
     * lance. Dans l'application c'est le cas -- {@code Application.launch} precede la
     * construction du contexte -- et il faut reproduire cet ordre ici.</p>
     */
    static {
        startToolkit();
    }

    @SpringBootApplication
    @Import({CoreConfiguration.class, AiConfiguration.class})
    static class TestApp {
    }

    @Autowired
    private List<AppView> views;

    @Test
    @DisplayName("chaque section se construit, avec sa feuille de style, sans rien casser")
    void everyViewBuilds() throws Exception {
        List<String> failures = new ArrayList<>();
        List<String> built = new ArrayList<>();
        CountDownLatch done = new CountDownLatch(1);

        Platform.runLater(() -> {
            try {
                StackPane root = new StackPane();
                Scene scene = new Scene(root, 1400, 900);
                scene.getStylesheets().add(
                        MainWindow.class.getResource("app.css").toExternalForm());

                for (AppView view : views) {
                    try {
                        Node content = view.create();
                        // Attache reellement le noeud : c'est la mise en page qui revele
                        // les contraintes impossibles, pas la construction.
                        root.getChildren().setAll(content);
                        root.applyCss();
                        root.layout();
                        built.add(view.title());
                    } catch (Exception e) {
                        failures.add(view.title() + " : " + e);
                    }
                }
            } finally {
                done.countDown();
            }
        });

        assertThat(done.await(60, TimeUnit.SECONDS)).as("construction des vues").isTrue();
        assertThat(failures).isEmpty();
        assertThat(built).hasSameSizeAs(views);
        assertThat(views).as("toutes les sections doivent etre declarees").hasSizeGreaterThanOrEqualTo(9);
    }

    /** Sans environnement graphique, ce test n'a rien a verifier : il s'abstient. */
    private static void startToolkit() {
        try {
            CountDownLatch ready = new CountDownLatch(1);
            Platform.startup(ready::countDown);
            Assumptions.assumeTrue(ready.await(30, TimeUnit.SECONDS), "toolkit JavaFX indisponible");
        } catch (IllegalStateException alreadyStarted) {
            // Deja demarre par un autre test : parfait.
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            Assumptions.abort("demarrage du toolkit interrompu");
        } catch (UnsupportedOperationException | Error e) {
            Assumptions.abort("environnement sans affichage : " + e.getMessage());
        }
    }
}
