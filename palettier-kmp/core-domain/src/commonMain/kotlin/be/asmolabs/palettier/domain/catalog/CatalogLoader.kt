package be.asmolabs.palettier.domain.catalog

import be.asmolabs.palettier.domain.color.Rgb
import be.asmolabs.palettier.domain.paint.Opacity
import be.asmolabs.palettier.domain.paint.Paint

/**
 * Construit le catalogue a partir des fichiers de gamme.
 *
 * <p>Une gamme = un fichier. Ajouter une marque ne demande donc aucune modification du
 * code metier, seulement un fichier de plus dans {@code catalog/brands/} : la liste des
 * fichiers est relevee a la compilation, pas ecrite a la main.</p>
 *
 * <p>Seules les colonnes {@code brand}, {@code name} et {@code pigments} sont
 * obligatoires : l'opacite et la teinte sont deduites des pigments quand le fichier ne
 * les precise pas, et les tubes ainsi completes sont marques comme approximatifs.</p>
 */
/**
 * Un fichier de gamme : son nom, qui sert a situer une erreur de lecture, et son texte.
 *
 * <p>Le texte est compile dans l'application plutot que lu a l'execution. Ce n'est pas
 * une commodite : core-domain n'admet que du Kotlin multiplateforme pur, et il n'existe
 * pas de classpath sur un telephone. Une constante generee, elle, marche partout.</p>
 */
class BrandFile(val name: String, val content: String)

class CatalogLoader(
    private val pigments: PigmentIndex = PigmentIndex(),
    private val brands: List<BrandFile> = CatalogFiles.BRANDS,
) {

    /** Ce que le chargement a trouve, et ce qu'il n'a pas su rattacher. */
    data class Loaded(
        val paints: List<Paint>,
        /** Codes cites par les gammes mais absents de pigments.csv : liste de travail. */
        val unknownPigments: List<String>,
    )

    fun load(): List<Paint> = loadDetailed().paints

    fun loadDetailed(): Loaded {
        val paints = mutableListOf<Paint>()
        val cited = LinkedHashSet<String>()

        for (brand in brands) {
            for (row in Csv.read(brand.content, brand.name)) {
                val paint = toPaint(row)
                cited += paint.pigments
                paints += paint
            }
        }
        return Loaded(paints.toList(), pigments.unknownAmong(cited))
    }

    private fun toPaint(row: Csv.Row): Paint {
        val pigmentCodes = parsePigments(row["pigments"])
        check(pigmentCodes.isNotEmpty()) { "Aucun pigment declare en ${row.location()}" }

        val declaredHex = row.getOrDefault("hex", "")
        val derived = declaredHex.isBlank()
        val hex = if (derived) pigments.colorFor(pigmentCodes).toHex() else declaredHex

        val opacity = row.getOrDefault("opacity", "")
            .takeIf { it.isNotBlank() }
            ?.let { Opacity.valueOf(it) }
            ?: pigments.opacityFor(pigmentCodes)

        val tint = row.getOrDefault("tint", "")
            .takeIf { it.isNotBlank() }
            ?: pigments.tintFor(pigmentCodes, Rgb.ofHex(hex))?.toHex()

        return Paint(
            brand = row["brand"],
            name = row["name"],
            code = row.getOrDefault("code", ""),
            // Colonne facultative : seules les gammes renumerotees par leur fabricant la remplissent.
            legacyCode = row.getOrDefault("legacy", ""),
            pigments = pigmentCodes,
            hexColor = hex,
            tintHex = tint,
            opacity = opacity,
            dryingClass = pigments.dryingClassFor(pigmentCodes),
            tintingStrength = pigments.tintingStrengthFor(pigmentCodes),
            colorDerived = derived,
            // Colonne facultative : une gamme qui ne la renseigne pas est tenue pour non verifiee.
            pigmentsVerified = row.getOrDefault("verified", "false").toBooleanStrictOrNull() ?: false,
        )
    }

    /** Les pigments sont listes separes par des espaces : "PBr7 PBk9". */
    private fun parsePigments(cell: String): Set<String> =
        cell.split(" ", "\t").map { it.trim() }.filter { it.isNotEmpty() }.toCollection(LinkedHashSet())
}
