package be.asmolabs.palettier.ui.export

import be.asmolabs.palettier.domain.color.Rgb
import be.asmolabs.palettier.domain.plan.PaintingPlan
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import org.apache.pdfbox.pdmodel.PDDocument
import org.apache.pdfbox.pdmodel.PDPage
import org.apache.pdfbox.pdmodel.PDPageContentStream
import org.apache.pdfbox.pdmodel.common.PDRectangle
import org.apache.pdfbox.pdmodel.font.PDType1Font
import org.apache.pdfbox.pdmodel.font.Standard14Fonts
import org.apache.pdfbox.pdmodel.graphics.color.PDColor
import org.apache.pdfbox.pdmodel.graphics.color.PDDeviceRGB
import java.io.ByteArrayOutputStream

/**
 * La mise sur papier du bureau : PDFBox, comme du temps de JavaFX.
 *
 * <p>La feuille se suffit a elle-meme parce que c'est son seul interet : a l'etabli on ne
 * consulte pas un ecran les mains grasses de diluant. Les pastilles y sont dessinees, les
 * dosages ecrits en toutes lettres, et l'ecart mesure indique pour qu'on sache d'avance ou
 * la palette va manquer.</p>
 */
class PdfPlanExporter : PlanExporter {

    override suspend fun render(plan: PaintingPlan): ByteArray = withContext(Dispatchers.IO) {
        Sheet().use { sheet ->
            sheet.text("Plan de peinture", sheet.bold, 18f, 0f)
            sheet.y -= 4
            sheet.text(plan.subject.ifBlank { "Sujet decrit par photo" }, sheet.regular, 11f, 0f)
            sheet.text("Palette : ${plan.paletteName}", sheet.regular, 9f, 0f)
            sheet.y -= 8

            if (plan.approach.isNotBlank()) {
                sheet.paragraph(plan.approach, 9.5f, 0f)
                sheet.y -= 8
            }

            plan.zones.forEach { zone -> writeZone(sheet, zone) }

            sheet.ensure(3 * LINE)
            sheet.y -= 6
            sheet.paragraph(
                "Les couleurs visees ont ete choisies pour ce sujet ; les dosages et l'ecart " +
                    "indique sont calcules a partir des tubes de la palette. Un ecart sous 2 est " +
                    "invisible a l'oeil, au-dela de 10 la palette ne permet pas d'atteindre la " +
                    "teinte. Les pastilles imprimees ne valent que ce que vaut votre imprimante : " +
                    "fiez-vous aux dosages.",
                8f,
                0f,
            )

            sheet.bytes()
        }
    }

    private fun writeZone(sheet: Sheet, zone: PaintingPlan.Zone) {
        // Une zone tient d'un bloc ou passe a la page suivante : couper un degrade en deux
        // pages le rend illisible.
        sheet.ensure((zone.layers().size + 3) * LINE)
        sheet.y -= 6

        val title = if (zone.material.isBlank()) zone.name else "${zone.name}  -  ${zone.material}"
        sheet.text(title, sheet.bold, 12f, 0f)

        if (zone.note.isNotBlank()) sheet.paragraph(zone.note, 8.5f, 0f)
        sheet.y -= 2

        zone.layers().forEach { writeLayer(sheet, it) }

        if (zone.accents.isNotEmpty()) {
            sheet.ensure((zone.accents.size + 1) * 2 * LINE)
            sheet.y -= 4
            sheet.text("Variations locales", sheet.bold, 9.5f, 0f)
            zone.accents.forEach { writeLayer(sheet, it) }
        }
    }

    /** Une ligne de couche : pastilles visee et obtenue, dosage, ecart. */
    private fun writeLayer(sheet: Sheet, layer: PaintingPlan.Layer) {
        sheet.ensure(2 * LINE)
        val top = sheet.y + SWATCH - 3
        sheet.swatch(layer.target, 0f, top)
        sheet.swatch(layer.achieved, SWATCH + 3, top)

        sheet.text(
            layer.role.padEnd(22) + " " + (layer.recipe?.describe() ?: "palette vide"),
            sheet.regular,
            9.5f,
            2 * SWATCH + 12,
        )
        sheet.text(
            "%s  -  %s  -  ecart %.1f, %s".format(
                layer.target.toHex(),
                layer.technique.ifBlank { "technique non precisee" },
                layer.deltaE,
                layer.reachability(),
            ),
            sheet.regular,
            8f,
            2 * SWATCH + 12,
        )
        sheet.y -= 3
    }

    /** Etat d'ecriture : la page courante, et ou l'on en est verticalement. */
    private class Sheet : AutoCloseable {

        val regular: PDType1Font = PDType1Font(Standard14Fonts.FontName.HELVETICA)
        val bold: PDType1Font = PDType1Font(Standard14Fonts.FontName.HELVETICA_BOLD)

        private val document = PDDocument()
        private var content: PDPageContentStream? = null
        var y: Float = 0f

        init {
            newPage()
        }

        fun newPage() {
            content?.close()
            val page = PDPage(PAGE)
            document.addPage(page)
            content = PDPageContentStream(document, page)
            y = PAGE.height - MARGIN
        }

        /** Passe a la page suivante si la place manque pour le bloc annonce. */
        fun ensure(height: Float) {
            if (y - height < MARGIN) newPage()
        }

        fun text(value: String, font: PDType1Font, size: Float, indent: Float) {
            val stream = content ?: return
            stream.beginText()
            stream.setFont(font, size)
            stream.newLineAtOffset(MARGIN + indent, y)
            stream.showText(sanitise(value))
            stream.endText()
            y -= LINE
        }

        /** Ecrit un paragraphe en le coupant a la largeur utile. */
        fun paragraph(value: String, size: Float, indent: Float) {
            wrap(value, size, PAGE.width - 2 * MARGIN - indent).forEach { line ->
                ensure(LINE)
                text(line, regular, size, indent)
            }
        }

        fun swatch(color: Rgb, x: Float, top: Float) {
            val stream = content ?: return
            stream.setNonStrokingColor(color.toPdf())
            stream.addRect(MARGIN + x, top - SWATCH, SWATCH, SWATCH)
            stream.fill()
            stream.setNonStrokingColor(PDColor(floatArrayOf(0f, 0f, 0f), PDDeviceRGB.INSTANCE))
        }

        fun wrap(value: String, size: Float, width: Float): List<String> {
            val lines = mutableListOf<String>()
            var current = StringBuilder()
            for (word in sanitise(value).split(Regex("\\s+")).filter { it.isNotEmpty() }) {
                val candidate = if (current.isEmpty()) word else "$current $word"
                if (regular.getStringWidth(candidate) / 1000 * size > width && current.isNotEmpty()) {
                    lines += current.toString()
                    current = StringBuilder(word)
                } else {
                    current = StringBuilder(candidate)
                }
            }
            if (current.isNotEmpty()) lines += current.toString()
            return lines
        }

        /**
         * Rend le document complet.
         *
         * <p>PDFBox refuse d'ecrire tant qu'un flux de contenu reste ouvert : il faut
         * fermer la derniere page avant d'enregistrer, et non l'inverse.</p>
         */
        fun bytes(): ByteArray {
            closeContent()
            return ByteArrayOutputStream().also { document.save(it) }.toByteArray()
        }

        private fun closeContent() {
            content?.close()
            content = null
        }

        override fun close() {
            closeContent()
            document.close()
        }
    }

    private companion object {
        val PAGE: PDRectangle = PDRectangle.A4
        const val MARGIN = 45f
        const val SWATCH = 13f
        const val LINE = 13.5f

        fun Rgb.toPdf(): PDColor =
            PDColor(floatArrayOf(r.toFloat(), g.toFloat(), b.toFloat()), PDDeviceRGB.INSTANCE)

        /**
         * Les polices standard du PDF ne codent que le latin occidental. Un caractere
         * absent ferait echouer l'ecriture entiere : on remplace plutot que de perdre le
         * document.
         */
        fun sanitise(value: String): String = value
            .map { c ->
                when {
                    c == '’' -> '\''
                    c == '\n' -> ' '
                    c.code < 32 -> ' '
                    c.code > 255 -> '?'
                    else -> c
                }
            }
            .joinToString("")
    }
}
