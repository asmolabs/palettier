package be.asmolabs.palettier.ai;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import java.io.BufferedReader;
import java.io.InputStreamReader;
import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.charset.StandardCharsets;
import java.time.Duration;
import java.util.ArrayList;
import java.util.List;
import java.util.function.Consumer;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

/**
 * Dialogue avec le serveur Ollama local : ce qui est installe, et comment en installer
 * davantage.
 *
 * <p>Ollama publie tout cela sans authentification, sur la machine meme. Les autres
 * moteurs exposent des catalogues lies a un compte : pour eux, le nom du modele reste
 * une saisie libre.</p>
 */
@Component
public class ModelCatalog {

    private static final Logger log = LoggerFactory.getLogger(ModelCatalog.class);
    private static final Duration SHORT_TIMEOUT = Duration.ofSeconds(4);

    /** Un telechargement de modele se compte en gigaoctets : aucune limite de duree. */
    private static final Duration PULL_TIMEOUT = Duration.ofHours(2);

    private final String provider;
    private final String baseUrl;
    private final ObjectMapper json = new ObjectMapper();

    ModelCatalog(@Value("${spring.ai.model.chat:none}") String provider,
                 @Value("${spring.ai.ollama.base-url:http://localhost:11434}") String baseUrl) {
        this.provider = provider;
        this.baseUrl = baseUrl.replaceAll("/+$", "");
    }

    /**
     * Un modele installe.
     *
     * @param capabilities ce que le modele sait faire. La presence de {@code vision} est
     *                     decisive : sans elle, l'assistant ne peut pas lire vos photos.
     */
    public record ModelInfo(String name, long sizeBytes, List<String> capabilities) {

        public boolean supportsVision() {
            return capabilities.contains("vision");
        }

        public String sizeLabel() {
            return sizeBytes <= 0 ? "" : "%.1f Go".formatted(sizeBytes / 1e9);
        }
    }

    /** Avancement d'un telechargement. */
    public record PullProgress(String status, long completed, long total) {

        /** Fraction telechargee, ou -1 quand le serveur ne l'indique pas encore. */
        public double fraction() {
            return total > 0 ? (double) completed / total : -1;
        }
    }

    public String provider() {
        return provider;
    }

    public String baseUrl() {
        return baseUrl;
    }

    /** Vrai quand le moteur actif permet de lister et d'installer des modeles. */
    public boolean isManageable() {
        return "ollama".equalsIgnoreCase(provider);
    }

    /** Vrai si le serveur repond. */
    public boolean reachable() {
        return isManageable() && get("/api/tags") != null;
    }

    /** Modeles installes, avec leurs capacites. Liste vide si le serveur ne repond pas. */
    public List<ModelInfo> installed() {
        JsonNode tags = isManageable() ? get("/api/tags") : null;
        if (tags == null) {
            return List.of();
        }
        List<ModelInfo> models = new ArrayList<>();
        for (JsonNode model : tags.path("models")) {
            String name = model.path("name").asText("");
            if (!name.isBlank()) {
                models.add(new ModelInfo(name, model.path("size").asLong(0), capabilitiesOf(name)));
            }
        }
        models.sort((a, b) -> a.name().compareToIgnoreCase(b.name()));
        return List.copyOf(models);
    }

    /** Noms seuls, pour les listes deroulantes. */
    public List<String> available() {
        return installed().stream().map(ModelInfo::name).toList();
    }

    private List<String> capabilitiesOf(String model) {
        JsonNode details = post("/api/show", "{\"model\":\"" + model + "\"}");
        if (details == null) {
            return List.of();
        }
        List<String> capabilities = new ArrayList<>();
        details.path("capabilities").forEach(node -> capabilities.add(node.asText()));
        return List.copyOf(capabilities);
    }

    /**
     * Telecharge un modele et rend compte de l'avancement au fur et a mesure.
     *
     * <p>Ollama repond en flux : une ligne JSON par etape. On les transmet telles quelles
     * plutot que d'attendre la fin, sans quoi l'ecran resterait fige pendant plusieurs
     * gigaoctets.</p>
     *
     * @param onProgress recoit chaque etape ; appele depuis le fil qui invoque la methode
     * @return vrai si le telechargement s'est acheve
     */
    public boolean pull(String model, Consumer<PullProgress> onProgress) {
        if (!isManageable() || model == null || model.isBlank()) {
            return false;
        }
        HttpRequest request = HttpRequest.newBuilder()
                .uri(URI.create(baseUrl + "/api/pull"))
                .timeout(PULL_TIMEOUT)
                .header("Content-Type", "application/json")
                .POST(HttpRequest.BodyPublishers.ofString(
                        "{\"model\":\"" + model.trim() + "\",\"stream\":true}", StandardCharsets.UTF_8))
                .build();

        try (HttpClient client = HttpClient.newBuilder().connectTimeout(SHORT_TIMEOUT).build()) {
            HttpResponse<java.io.InputStream> response =
                    client.send(request, HttpResponse.BodyHandlers.ofInputStream());
            if (response.statusCode() != 200) {
                onProgress.accept(new PullProgress("refuse par le serveur (" + response.statusCode() + ")", 0, 0));
                return false;
            }

            boolean success = false;
            try (BufferedReader reader = new BufferedReader(
                    new InputStreamReader(response.body(), StandardCharsets.UTF_8))) {
                String line;
                while ((line = reader.readLine()) != null) {
                    if (line.isBlank()) {
                        continue;
                    }
                    JsonNode step = json.readTree(line);
                    if (step.hasNonNull("error")) {
                        onProgress.accept(new PullProgress(step.get("error").asText(), 0, 0));
                        return false;
                    }
                    String status = step.path("status").asText("");
                    onProgress.accept(new PullProgress(status,
                            step.path("completed").asLong(0), step.path("total").asLong(0)));
                    success = "success".equals(status);
                }
            }
            return success;
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            return false;
        } catch (Exception e) {
            log.debug("Telechargement du modele {} impossible", model, e);
            onProgress.accept(new PullProgress("echec : " + e.getMessage(), 0, 0));
            return false;
        }
    }

    // --- Acces HTTP --------------------------------------------------------

    private JsonNode get(String path) {
        return call(HttpRequest.newBuilder().uri(URI.create(baseUrl + path)).GET());
    }

    private JsonNode post(String path, String body) {
        return call(HttpRequest.newBuilder()
                .uri(URI.create(baseUrl + path))
                .header("Content-Type", "application/json")
                .POST(HttpRequest.BodyPublishers.ofString(body, StandardCharsets.UTF_8)));
    }

    private JsonNode call(HttpRequest.Builder builder) {
        try (HttpClient client = HttpClient.newBuilder().connectTimeout(SHORT_TIMEOUT).build()) {
            HttpResponse<String> response = client.send(
                    builder.timeout(SHORT_TIMEOUT).build(), HttpResponse.BodyHandlers.ofString());
            return response.statusCode() == 200 ? json.readTree(response.body()) : null;
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            return null;
        } catch (Exception e) {
            log.debug("Serveur Ollama injoignable sur {}", baseUrl, e);
            return null;
        }
    }
}
