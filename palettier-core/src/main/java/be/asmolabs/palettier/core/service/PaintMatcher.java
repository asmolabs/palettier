package be.asmolabs.palettier.core.service;

import be.asmolabs.palettier.core.domain.OilPaint;
import java.text.Normalizer;
import java.util.Arrays;
import java.util.Comparator;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Optional;
import java.util.Set;
import org.springframework.stereotype.Service;

/**
 * Retrouve un tube du catalogue a partir d'un libelle approximatif.
 *
 * <p>Ce qu'on lit sur un tube ou ce qu'un modele croit y lire ne correspond jamais
 * exactement a une fiche : accents perdus, ponctuation differente, marque abregee,
 * mot en trop. La comparaison se fait donc sur des mots normalises, et le resultat
 * est accompagne d'une confiance -- parce qu'associer le mauvais tube est pire que
 * n'en associer aucun.</p>
 */
@Service
public class PaintMatcher {

    /** En deca, l'association est trop douteuse pour etre proposee. */
    private static final double MINIMUM_CONFIDENCE = 0.45;

    /** Mots qui n'aident pas a distinguer deux tubes. */
    private static final Set<String> NOISE = Set.of(
            "oil", "color", "colour", "couleur", "huile", "paint", "peinture",
            "artists", "artist", "professional", "fine", "ml", "tube", "series", "serie");

    /**
     * Une association proposee.
     *
     * @param confidence de 0 a 1 ; 1 signifie que tous les mots du libelle ont ete retrouves
     */
    public record Match(OilPaint paint, double confidence, String source) {

        public boolean isReliable() {
            return confidence >= 0.7;
        }
    }

    /**
     * Cherche le tube correspondant a un libelle libre.
     *
     * @param label     ce qui est lu, par exemple "W&N Burnt Umber 37ml"
     * @param catalogue les tubes parmi lesquels chercher
     */
    public Optional<Match> match(String label, List<OilPaint> catalogue) {
        Set<String> wanted = words(label);
        if (wanted.isEmpty()) {
            return Optional.empty();
        }

        return catalogue.stream()
                .map(paint -> new Match(paint, score(wanted, paint), label))
                .filter(match -> match.confidence() >= MINIMUM_CONFIDENCE)
                .max(Comparator.comparingDouble(Match::confidence));
    }

    /**
     * Part des mots du libelle retrouves dans la fiche.
     *
     * <p>Les mots de la marque sont ecartes avant la comparaison, des deux cotes. Sans
     * cela "Winsor & Newton" seul designerait "Winsor Lemon" avec assurance, puisque le
     * mot se retrouve dans le nom : une marque ne doit jamais suffire a choisir un tube
     * au hasard dans sa gamme.</p>
     */
    private static double score(Set<String> wanted, OilPaint paint) {
        Set<String> brandWords = words(paint.getBrand());
        Set<String> nameWords = distinctive(words(paint.getName()), brandWords);
        // Si le libelle ne contient que des mots de la marque, il ne designe rien.
        Set<String> askedFor = new LinkedHashSet<>(wanted);
        askedFor.removeAll(brandWords);
        if (askedFor.isEmpty() || nameWords.isEmpty()) {
            return 0;
        }
        long nameHits = askedFor.stream().filter(nameWords::contains).count();
        if (nameHits == 0) {
            return 0;
        }

        double nameScore = (double) nameHits / nameWords.size();
        boolean brandNamed = wanted.stream().anyMatch(brandWords::contains);
        // Un mot du libelle qui ne correspond a rien fait douter, sans disqualifier.
        double noise = 1.0 - Math.min(0.3, (askedFor.size() - nameHits) * 0.1);

        return Math.min(1.0, nameScore * noise + (brandNamed ? 0.2 : 0));
    }

    /** Ce qui reste d'un ensemble de mots une fois la marque retiree. */
    private static Set<String> distinctive(Set<String> words, Set<String> brandWords) {
        Set<String> remaining = new LinkedHashSet<>(words);
        remaining.removeAll(brandWords);
        // Un tube dont le nom n'est que la marque garde son nom, faute de mieux.
        return remaining.isEmpty() ? words : remaining;
    }

    /** Mots significatifs, sans accents ni ponctuation. */
    private static Set<String> words(String text) {
        if (text == null || text.isBlank()) {
            return Set.of();
        }
        String plain = Normalizer.normalize(text, Normalizer.Form.NFD)
                .replaceAll("\\p{M}", "")
                .toLowerCase()
                .replaceAll("[^a-z0-9]+", " ");
        return Arrays.stream(plain.split(" "))
                .filter(word -> word.length() > 1)
                .filter(word -> !NOISE.contains(word))
                .collect(LinkedHashSet::new, Set::add, Set::addAll);
    }
}
