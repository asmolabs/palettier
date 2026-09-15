package be.asmolabs.palettier.core.catalog;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.io.UncheckedIOException;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import org.springframework.core.io.Resource;

/**
 * Lecteur de fichiers point-virgule.
 *
 * <p>Les donnees du catalogue sont du texte tabulaire edite a la main : un format
 * lisible dans un tableur, versionnable, et qu'un peintre peut corriger sans
 * recompiler. Le separateur est le point-virgule parce que les noms de couleurs
 * contiennent des virgules ("Winsor Blue, Green Shade").</p>
 */
final class Csv {

    private static final char SEPARATOR = ';';
    private static final String COMMENT = "#";

    private Csv() {
    }

    /** Une ligne, accessible par nom de colonne. */
    record Row(Map<String, String> values, String origin, int lineNumber) {

        String get(String column) {
            String value = values.get(column);
            if (value == null) {
                throw new IllegalStateException("Colonne '%s' absente dans %s".formatted(column, origin));
            }
            return value;
        }

        /** Valeur de la colonne, ou {@code fallback} si la colonne est absente ou vide. */
        String getOrDefault(String column, String fallback) {
            String value = values.get(column);
            return value == null || value.isBlank() ? fallback : value;
        }

        String location() {
            return origin + ":" + lineNumber;
        }
    }

    /** Lit un fichier ; la premiere ligne non vide et non commentee porte les en-tetes. */
    static List<Row> read(Resource resource) {
        String origin = resource.getFilename() == null ? resource.getDescription() : resource.getFilename();
        List<Row> rows = new ArrayList<>();

        try (BufferedReader reader = new BufferedReader(
                new InputStreamReader(resource.getInputStream(), StandardCharsets.UTF_8))) {

            List<String> headers = null;
            String line;
            int lineNumber = 0;

            while ((line = reader.readLine()) != null) {
                lineNumber++;
                if (line.isBlank() || line.stripLeading().startsWith(COMMENT)) {
                    continue;
                }
                List<String> cells = split(line);
                if (headers == null) {
                    headers = cells;
                    continue;
                }
                Map<String, String> values = new LinkedHashMap<>();
                for (int i = 0; i < headers.size(); i++) {
                    values.put(headers.get(i), i < cells.size() ? cells.get(i) : "");
                }
                rows.add(new Row(values, origin, lineNumber));
            }
        } catch (IOException e) {
            throw new UncheckedIOException("Lecture impossible de " + origin, e);
        }
        return rows;
    }

    private static List<String> split(String line) {
        List<String> cells = new ArrayList<>();
        int start = 0;
        for (int i = 0; i < line.length(); i++) {
            if (line.charAt(i) == SEPARATOR) {
                cells.add(line.substring(start, i).trim());
                start = i + 1;
            }
        }
        cells.add(line.substring(start).trim());
        return cells;
    }
}
