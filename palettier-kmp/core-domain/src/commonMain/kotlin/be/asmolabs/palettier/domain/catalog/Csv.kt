package be.asmolabs.palettier.domain.catalog

/**
 * Lecteur de fichiers point-virgule.
 *
 * <p>Les donnees du catalogue sont du texte tabulaire edite a la main : un format lisible
 * dans un tableur, versionnable, et qu'un peintre peut corriger sans recompiler. Le
 * separateur est le point-virgule parce que les noms de couleurs contiennent des virgules
 * ("Winsor Blue, Green Shade").</p>
 *
 * <p>Difference avec la version Java : on recoit le texte, pas une ressource. Le domaine
 * ne sait pas lire un fichier -- il n'y a pas de classpath sur un telephone -- et c'est
 * tres bien ainsi : le lecteur devient testable sans fichier d'essai.</p>
 */
object Csv {

    private const val SEPARATOR = ';'
    private const val COMMENT = "#"

    /** Une ligne, accessible par nom de colonne. */
    class Row(
        private val values: Map<String, String>,
        private val origin: String,
        private val lineNumber: Int,
    ) {

        operator fun get(column: String): String = values[column]
            ?: error("Colonne '$column' absente dans $origin")

        /** Valeur de la colonne, ou [fallback] si la colonne est absente ou vide. */
        fun getOrDefault(column: String, fallback: String): String {
            val value = values[column]
            return if (value.isNullOrBlank()) fallback else value
        }

        fun location(): String = "$origin:$lineNumber"
    }

    /** Lit un texte ; la premiere ligne non vide et non commentee porte les en-tetes. */
    fun read(text: String, origin: String): List<Row> {
        val rows = mutableListOf<Row>()
        var headers: List<String>? = null

        text.lineSequence().forEachIndexed { index, line ->
            if (line.isBlank() || line.trimStart().startsWith(COMMENT)) return@forEachIndexed
            val cells = split(line)
            if (headers == null) {
                headers = cells
                return@forEachIndexed
            }
            val columns = headers
            val values = LinkedHashMap<String, String>()
            for (i in columns.indices) {
                values[columns[i]] = if (i < cells.size) cells[i] else ""
            }
            rows += Row(values, origin, index + 1)
        }
        return rows
    }

    private fun split(line: String): List<String> = line.split(SEPARATOR).map { it.trim() }
}
