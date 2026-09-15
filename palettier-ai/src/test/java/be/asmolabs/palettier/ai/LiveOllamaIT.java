package be.asmolabs.palettier.ai;

import static org.assertj.core.api.Assertions.assertThat;

import be.asmolabs.palettier.core.config.CoreConfiguration;
import be.asmolabs.palettier.core.domain.Palette;
import be.asmolabs.palettier.core.plan.PaintingPlan;
import be.asmolabs.palettier.core.service.PaletteService;
import org.junit.jupiter.api.DisplayName;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.condition.EnabledIfEnvironmentVariable;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.context.annotation.Import;
import org.springframework.test.context.TestPropertySource;

/**
 * Appel reel contre un moteur local. Ne s'execute que si PALETTIER_AI_LIVE est definie :
 * un test qui depend d'un service externe n'a rien a faire dans la suite ordinaire.
 */
@SpringBootTest
@TestPropertySource(properties = {
        "spring.ai.model.chat=ollama",
        // Les starters embarquent aussi des modeles audio, image et embedding qui
        // tenteraient de s'instancier sans identifiants. On les coupe explicitement.
        "spring.ai.model.embedding=none",
        "spring.ai.model.image=none",
        "spring.ai.model.audio.speech=none",
        "spring.ai.model.audio.transcription=none",
        "spring.ai.model.moderation=none",
        "spring.ai.ollama.chat.options.model=${OLLAMA_MODEL:gemma4:e4b}",
        "spring.ai.ollama.chat.options.temperature=0.2",
        "spring.ai.ollama.chat.options.num-ctx=32768",
        "spring.ai.ollama.chat.options.num-predict=16384",
        "spring.jpa.hibernate.ddl-auto=create-drop"
})
@EnabledIfEnvironmentVariable(named = "PALETTIER_AI_LIVE", matches = "1")
class LiveOllamaIT {

    @SpringBootApplication
    @Import({CoreConfiguration.class, AiConfiguration.class})
    static class TestApp {
    }

    @Autowired
    private PaintingPlanService planner;

    @Autowired
    private PaletteService palettes;

    @Test
    @DisplayName("un plan reel est produit, et chaque couche porte un melange calcule")
    void producesAUsablePlan() {
        assertThat(planner.isAvailable()).isTrue();

        Palette zorn = palettes.findAll().stream()
                .filter(p -> p.getName().startsWith("Palette Zorn"))
                .findFirst()
                .orElseThrow();

        PaintingPlan plan = planner.plan(
                "Buste de grognard napoleonien : visage burine, mains, col de chemise blanc", zorn, 3);

        System.out.println("=== approche : " + plan.approach());
        plan.zones().forEach(zone -> {
            System.out.println("--- zone : " + zone.name() + " (" + zone.material() + ")");
            zone.layers().forEach(l -> System.out.printf("    %-8s %s -> %s  ecart %.1f  [%s]  %s%n",
                    l.role(), l.target().toHex(), l.achieved().toHex(), l.deltaE(),
                    l.technique(), l.recipe() == null ? "-" : l.recipe().describe()));
        });

        assertThat(plan.zones()).isNotEmpty();
        assertThat(plan.zones()).allSatisfy(zone ->
                assertThat(zone.layers()).allSatisfy(layer -> {
                    assertThat(layer.target()).isNotNull();
                    assertThat(layer.recipe()).as("chaque couche doit porter un melange calcule").isNotNull();
                    assertThat(layer.recipe().parts()).isNotEmpty();
                }));
    }

    @Test
    @DisplayName("une photo jointe est bien transmise et le decoupage en zones en tient compte")
    void usesTheAttachedPhoto() throws Exception {
        // L'image de reference n'est pas versionnee : sans elle, ce test n'a rien a prouver.
        String path = System.getenv().getOrDefault("PALETTIER_TEST_IMAGE", "/tmp/buste-test.png");
        java.nio.file.Path file = java.nio.file.Path.of(path);
        org.junit.jupiter.api.Assumptions.assumeTrue(java.nio.file.Files.exists(file),
                "image de test absente : " + path);
        byte[] image = java.nio.file.Files.readAllBytes(file);

        Palette zorn = palettes.findAll().stream()
                .filter(p -> p.getName().startsWith("Palette Zorn"))
                .findFirst()
                .orElseThrow();

        PaintingPlan plan = planner.plan(
                "", zorn, 3,
                new PhotoInput(image, "la piece a peindre, etat actuel"),
                java.util.List.of());

        System.out.println("=== avec photo, approche : " + plan.approach());
        plan.zones().forEach(zone -> {
            System.out.println("--- zone : " + zone.name() + " (" + zone.material() + ")");
            zone.layers().forEach(l -> System.out.printf("    %-8s %s -> %s  ecart %.1f  %s%n",
                    l.role(), l.target().toHex(), l.achieved().toHex(), l.deltaE(),
                    l.recipe() == null ? "-" : l.recipe().describe()));
        });

        assertThat(plan.zones()).isNotEmpty();
        assertThat(plan.zones()).allSatisfy(zone ->
                assertThat(zone.layers()).allSatisfy(layer ->
                        assertThat(layer.recipe()).isNotNull()));
    }
}
