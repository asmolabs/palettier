package be.asmolabs.palettier.ai;

import be.asmolabs.palettier.core.domain.OilPaint;
import be.asmolabs.palettier.core.image.Photos;
import be.asmolabs.palettier.core.service.PaintMatcher;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.ai.chat.client.ChatClient;
import org.springframework.ai.chat.prompt.ChatOptions;
import org.springframework.beans.BeansException;
import org.springframework.beans.factory.ObjectProvider;
import org.springframework.core.io.ByteArrayResource;
import org.springframework.stereotype.Service;
import org.springframework.util.MimeTypeUtils;

/**
 * Lit les etiquettes d'une photo de tubes pour constituer l'inventaire.
 *
 * <p>Saisir quatre cents cases a la main est decourageant ; photographier sa boite ne
 * coute rien. Le modele ne fait que <em>lire</em> : il ne decide pas ce que vous
 * possedez, il propose des libelles que l'application rapproche ensuite du catalogue,
 * avec une confiance. La derniere main reste au peintre.</p>
 */
@Service
public class TubeRecognitionService {

    private static final Logger log = LoggerFactory.getLogger(TubeRecognitionService.class);
    private static final int MAX_TOKENS = 4096;

    private static final String SYSTEM_PROMPT = """
            Tu lis les etiquettes de tubes de peinture sur une photo.

            Tu renvoies la liste des tubes visibles, un libelle par tube, en recopiant ce
            qui est ecrit : marque puis nom de la couleur. Par exemple
            "Winsor & Newton Burnt Umber" ou "Abteilung 502 Shadow Brown".

            Ne devine pas ce que tu ne lis pas. Un tube dont l'etiquette est masquee, floue
            ou de dos ne doit pas figurer dans la liste : mieux vaut en oublier un que
            d'en inventer un. N'ajoute ni volume, ni serie, ni commentaire.
            """;

    /** Ce que le modele renvoie : des libelles, rien de plus. */
    record TubeLabels(List<String> tubes) {
    }

    /**
     * Un tube lu, et le rapprochement propose.
     *
     * @param match vide quand aucune fiche du catalogue ne correspond assez bien
     */
    public record Identification(String label, Optional<PaintMatcher.Match> match) {
    }

    private final ObjectProvider<ChatClient.Builder> chatClientBuilder;
    private final PaintMatcher matcher;
    private final AiSettings settings;

    public TubeRecognitionService(ObjectProvider<ChatClient.Builder> chatClientBuilder,
                                  PaintMatcher matcher, AiSettings settings) {
        this.chatClientBuilder = chatClientBuilder;
        this.matcher = matcher;
        this.settings = settings;
    }

    public boolean isAvailable() {
        try {
            return chatClientBuilder.getIfAvailable() != null;
        } catch (BeansException e) {
            log.debug("Aucun moteur de conversation utilisable", e);
            return false;
        }
    }

    /**
     * @param photo     image des tubes, brute ; elle est reduite avant l'envoi
     * @param catalogue fiches parmi lesquelles chercher
     * @param model     modele a employer, ou {@code null} pour celui des reglages
     */
    public List<Identification> identify(byte[] photo, List<OilPaint> catalogue, String model) {
        ChatClient.Builder builder = chatClientBuilder.getIfAvailable();
        if (builder == null) {
            throw new IllegalStateException(
                    "Aucun moteur de conversation configure. Renseignez spring.ai.model.chat.");
        }

        String chosen = model != null && !model.isBlank() ? model : settings.model().orElse(null);
        ChatOptions.Builder<?> options = ChatOptions.builder().maxTokens(MAX_TOKENS);
        if (chosen != null) {
            options.model(chosen);
        }

        TubeLabels read;
        try {
            read = builder.defaultSystem(SYSTEM_PROMPT).build()
                    .prompt()
                    .options(options)
                    .user(spec -> {
                        spec.text("Quels tubes vois-tu sur cette photo ?");
                        spec.media(MimeTypeUtils.parseMimeType(Photos.MIME_TYPE),
                                new ByteArrayResource(Photos.prepare(photo)));
                    })
                    .call()
                    .entity(TubeLabels.class);
        } catch (RuntimeException e) {
            throw new IllegalStateException(
                    "Le modele n'a pas su lire cette photo. Verifiez qu'il sait traiter les images "
                    + "(etiquette Vision dans les parametres), et que les etiquettes sont lisibles. ("
                    + e.getClass().getSimpleName() + ")", e);
        }

        List<Identification> found = new ArrayList<>();
        for (String label : read.tubes() == null ? List.<String>of() : read.tubes()) {
            if (label != null && !label.isBlank()) {
                found.add(new Identification(label.trim(), matcher.match(label, catalogue)));
            }
        }
        log.info("{} tubes lus sur la photo, {} rapproches du catalogue",
                found.size(), found.stream().filter(i -> i.match().isPresent()).count());
        return List.copyOf(found);
    }
}
