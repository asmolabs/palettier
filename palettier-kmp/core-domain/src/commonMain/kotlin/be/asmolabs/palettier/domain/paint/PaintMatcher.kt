package be.asmolabs.palettier.domain.paint

import be.asmolabs.palettier.domain.color.Rgb
import be.asmolabs.palettier.domain.color.deltaE2000

/**
 * Retrouve un tube a partir d'un libelle libre : "W&N Burnt Umber 37ml".
 *
 * <p>Mieux vaut n'associer aucun tube qu'en associer un faux : une fiche mal reconnue se
 * propage ensuite dans les melanges et les plannings sans que rien ne le signale.</p>
 */
class PaintMatcher {

    /**
     * Une association proposee.
     *
     * @param confidence de 0 a 1 ; 1 signifie que tous les mots du libelle ont ete retrouves
     */
    data class Match(val paint: Paint, val confidence: Double, val source: String) {
        val isReliable: Boolean get() = confidence >= 0.7
    }

    /**
     * Cherche le tube correspondant a un libelle libre.
     *
     * @param label     ce qui est lu, par exemple "W&N Burnt Umber 37ml"
     * @param catalogue les tubes parmi lesquels chercher
     */
    fun match(label: String?, catalogue: List<Paint>): Match? {
        val wanted = words(label)
        if (wanted.isEmpty()) return null

        return catalogue
            .map { Match(it, score(wanted, it), label.orEmpty()) }
            .filter { it.confidence >= MINIMUM_CONFIDENCE }
            .maxByOrNull { it.confidence }
    }

    companion object {
        /** En deca, l'association est trop douteuse pour etre proposee. */
        private const val MINIMUM_CONFIDENCE = 0.45

        /** Mots qui n'aident pas a distinguer deux tubes. */
        private val NOISE = setOf(
            "oil", "color", "colour", "couleur", "huile", "paint", "peinture",
            "artists", "artist", "professional", "fine", "ml", "tube", "series", "serie",
        )

        /**
         * Repliement des accents.
         *
         * <p>Le Java passait par java.text.Normalizer, qui n'existe pas en
         * multiplateforme. Une table explicite fait le meme travail sur ce qui apparait
         * reellement dans des noms d'huiles -- francais, allemand, italien -- et a
         * l'avantage de se lire : on voit ce qui est couvert, et donc ce qui ne l'est pas.</p>
         */
        private val FOLDED = mapOf(
            'à' to 'a', 'á' to 'a', 'â' to 'a', 'ä' to 'a', 'ã' to 'a', 'å' to 'a',
            'è' to 'e', 'é' to 'e', 'ê' to 'e', 'ë' to 'e',
            'ì' to 'i', 'í' to 'i', 'î' to 'i', 'ï' to 'i',
            'ò' to 'o', 'ó' to 'o', 'ô' to 'o', 'ö' to 'o', 'õ' to 'o',
            'ù' to 'u', 'ú' to 'u', 'û' to 'u', 'ü' to 'u',
            'ç' to 'c', 'ñ' to 'n', 'ý' to 'y', 'ÿ' to 'y',
        )

        /**
         * Part des mots du libelle retrouves dans la fiche.
         *
         * <p>Les mots de la marque sont ecartes des deux cotes avant la comparaison. Sans
         * cela "Winsor & Newton" seul designerait "Winsor Lemon" avec assurance, puisque
         * le mot se retrouve dans le nom : une marque ne doit jamais suffire a choisir un
         * tube au hasard dans sa gamme.</p>
         */
        private fun score(wanted: Set<String>, paint: Paint): Double {
            val brandWords = words(paint.brand)
            val nameWords = distinctive(words(paint.name), brandWords)

            // Si le libelle ne contient que des mots de la marque, il ne designe rien.
            val askedFor = wanted - brandWords
            if (askedFor.isEmpty() || nameWords.isEmpty()) return 0.0

            val nameHits = askedFor.count { it in nameWords }
            if (nameHits == 0) return 0.0

            val nameScore = nameHits.toDouble() / nameWords.size
            val brandNamed = wanted.any { it in brandWords }
            // Un mot du libelle qui ne correspond a rien fait douter, sans disqualifier.
            val noise = 1.0 - minOf(0.3, (askedFor.size - nameHits) * 0.1)

            return minOf(1.0, nameScore * noise + if (brandNamed) 0.2 else 0.0)
        }

        /** Ce qui reste d'un ensemble de mots une fois la marque retiree. */
        private fun distinctive(words: Set<String>, brandWords: Set<String>): Set<String> {
            val remaining = words - brandWords
            // Un tube dont le nom n'est que la marque garde son nom, faute de mieux.
            return remaining.ifEmpty { words }
        }

        /** Mots significatifs, sans accents ni ponctuation. */
        private fun words(text: String?): Set<String> {
            if (text.isNullOrBlank()) return emptySet()
            val plain = buildString {
                for (c in text.lowercase()) {
                    val folded = FOLDED[c] ?: c
                    append(if (folded in 'a'..'z' || folded in '0'..'9') folded else ' ')
                }
            }
            return plain.split(' ')
                .filter { it.length > 1 && it !in NOISE }
                .toCollection(LinkedHashSet())
        }
    }
}

/**
 * Classe les huiles par ecart percu a une couleur cible : repond a la question
 * "quel tube de mon etagere se rapproche le plus de cette teinte ?".
 */
fun findClosest(target: Rgb, candidates: List<Paint>, limit: Int): List<PaintMatchByColour> =
    candidates
        .map { PaintMatchByColour(it, deltaE2000(target, it.color)) }
        .sortedBy { it.deltaE }
        .take(limit)

/** Une huile du catalogue et son ecart a une couleur cible. */
data class PaintMatchByColour(val paint: Paint, val deltaE: Double)
