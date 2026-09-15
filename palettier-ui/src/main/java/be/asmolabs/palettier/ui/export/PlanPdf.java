package be.asmolabs.palettier.ui.export;

import be.asmolabs.palettier.core.plan.PaintingPlan;
import be.asmolabs.palettier.core.color.Rgb;
import java.io.IOException;
import java.nio.file.Path;
import java.time.LocalDate;
import java.time.format.DateTimeFormatter;
import java.util.ArrayList;
import java.util.List;
import java.util.Locale;
import org.apache.pdfbox.pdmodel.PDDocument;
import org.apache.pdfbox.pdmodel.PDPage;
import org.apache.pdfbox.pdmodel.PDPageContentStream;
import org.apache.pdfbox.pdmodel.common.PDRectangle;
import org.apache.pdfbox.pdmodel.font.PDType1Font;
import org.apache.pdfbox.pdmodel.font.Standard14Fonts;
import org.apache.pdfbox.pdmodel.graphics.color.PDColor;
import org.apache.pdfbox.pdmodel.graphics.color.PDDeviceRGB;
import org.springframework.stereotype.Component;

/**
 * Met un plan de peinture sur papier.
 *
 * <p>Un plan sert a l'etabli, ou l'on ne consulte pas un ecran les mains grasses de
 * diluant. La feuille imprimee doit donc se suffire a elle-meme : les pastilles de
 * couleur y sont dessinees, les dosages ecrits en toutes lettres, et l'ecart mesure
 * indique pour que l'on sache d'avance ou la palette va manquer.</p>
 */
@Component
public class PlanPdf {

    private static final PDRectangle PAGE = PDRectangle.A4;
    private static final float MARGIN = 45;
    private static final float SWATCH = 13;
    private static final float LINE = 13.5f;

    private static final DateTimeFormatter DATE = DateTimeFormatter.ofPattern("d MMMM yyyy", Locale.FRENCH);

    private final PDType1Font regular = new PDType1Font(Standard14Fonts.FontName.HELVETICA);
    private final PDType1Font bold = new PDType1Font(Standard14Fonts.FontName.HELVETICA_BOLD);

    /** Etat d'ecriture : la page courante, et ou l'on en est verticalement. */
    private final class Sheet implements AutoCloseable {

        private final PDDocument document = new PDDocument();
        private PDPageContentStream content;
        private float y;

        Sheet() throws IOException {
            newPage();
        }

        void newPage() throws IOException {
            if (content != null) {
                content.close();
            }
            PDPage page = new PDPage(PAGE);
            document.addPage(page);
            content = new PDPageContentStream(document, page);
            y = PAGE.getHeight() - MARGIN;
        }

        /** Passe a la page suivante si la place manque pour le bloc annonce. */
        void ensure(float height) throws IOException {
            if (y - height < MARGIN) {
                newPage();
            }
        }

        void text(String value, PDType1Font font, float size, float indent) throws IOException {
            content.beginText();
            content.setFont(font, size);
            content.newLineAtOffset(MARGIN + indent, y);
            content.showText(sanitise(value));
            content.endText();
            y -= LINE;
        }

        /** Ecrit un paragraphe en le coupant a la largeur utile. */
        void paragraph(String value, float size, float indent) throws IOException {
            for (String line : wrap(value, size, PAGE.getWidth() - 2 * MARGIN - indent)) {
                ensure(LINE);
                text(line, regular, size, indent);
            }
        }

        void swatch(Rgb color, float x, float top) throws IOException {
            content.setNonStrokingColor(toPdf(color));
            content.addRect(MARGIN + x, top - SWATCH, SWATCH, SWATCH);
            content.fill();
            content.setNonStrokingColor(new PDColor(new float[]{0, 0, 0}, PDDeviceRGB.INSTANCE));
        }

        List<String> wrap(String value, float size, float width) throws IOException {
            List<String> lines = new ArrayList<>();
            StringBuilder current = new StringBuilder();
            for (String word : sanitise(value).split("\\s+")) {
                String candidate = current.isEmpty() ? word : current + " " + word;
                if (regular.getStringWidth(candidate) / 1000 * size > width && !current.isEmpty()) {
                    lines.add(current.toString());
                    current = new StringBuilder(word);
                } else {
                    current = new StringBuilder(candidate);
                }
            }
            if (!current.isEmpty()) {
                lines.add(current.toString());
            }
            return lines;
        }

        /**
         * Ferme le flux de la page courante et enregistre le document.
         *
         * <p>PDFBox refuse d'ecrire tant qu'un flux de contenu reste ouvert : il faut
         * fermer la derniere page avant de sauvegarder, et non l'inverse.</p>
         */
        void save(Path destination) throws IOException {
            closeContent();
            document.save(destination.toFile());
        }

        private void closeContent() throws IOException {
            if (content != null) {
                content.close();
                content = null;
            }
        }

        @Override
        public void close() throws IOException {
            closeContent();
            document.close();
        }
    }

    /**
     * Ecrit le plan dans un fichier PDF.
     *
     * @throws IOException si le fichier ne peut pas etre ecrit
     */
    public void write(PaintingPlan plan, Path destination) throws IOException {
        try (Sheet sheet = new Sheet()) {
            sheet.text("Plan de peinture", bold, 18, 0);
            sheet.y -= 4;
            sheet.text(plan.subject() == null || plan.subject().isBlank()
                    ? "Sujet decrit par photo" : plan.subject(), regular, 11, 0);
            sheet.text("Palette : " + plan.paletteName() + "   -   " + LocalDate.now().format(DATE),
                    regular, 9, 0);
            sheet.y -= 8;

            if (plan.approach() != null && !plan.approach().isBlank()) {
                sheet.paragraph(plan.approach(), 9.5f, 0);
                sheet.y -= 8;
            }

            for (PaintingPlan.Zone zone : plan.zones()) {
                writeZone(sheet, zone);
            }

            sheet.ensure(3 * LINE);
            sheet.y -= 6;
            sheet.paragraph("Les couleurs visees ont ete choisies pour ce sujet ; les dosages et "
                    + "l'ecart indique sont calcules a partir des tubes de la palette. Un ecart sous 2 "
                    + "est invisible a l'oeil, au-dela de 10 la palette ne permet pas d'atteindre la "
                    + "teinte. Les pastilles imprimees ne valent que ce que vaut votre imprimante : "
                    + "fiez-vous aux dosages.", 8, 0);

            sheet.save(destination);
        }
    }

    private void writeZone(Sheet sheet, PaintingPlan.Zone zone) throws IOException {
        // Une zone tient d'un bloc ou passe a la page suivante : couper un degrade en deux
        // pages le rend illisible.
        sheet.ensure((zone.layers().size() + 3) * LINE);
        sheet.y -= 6;

        String title = zone.material() == null || zone.material().isBlank()
                ? zone.name() : zone.name() + "  -  " + zone.material();
        sheet.text(title, bold, 12, 0);

        if (zone.note() != null && !zone.note().isBlank()) {
            sheet.paragraph(zone.note(), 8.5f, 0);
        }
        sheet.y -= 2;

        for (PaintingPlan.Layer layer : zone.layers()) {
            sheet.ensure(2 * LINE);
            float top = sheet.y + SWATCH - 3;
            sheet.swatch(layer.target(), 0, top);
            sheet.swatch(layer.achieved(), SWATCH + 3, top);

            sheet.text("%-11s %s".formatted(layer.role(),
                    layer.recipe() == null ? "palette vide" : layer.recipe().describe()),
                    regular, 9.5f, 2 * SWATCH + 12);
            sheet.text("%s  -  %s  -  ecart %.1f, %s".formatted(layer.target().toHex(),
                    layer.technique() == null || layer.technique().isBlank()
                            ? "technique non precisee" : layer.technique(),
                    layer.deltaE(), layer.reachability()),
                    regular, 8, 2 * SWATCH + 12);
            sheet.y -= 3;
        }
    }

    private static PDColor toPdf(Rgb color) {
        return new PDColor(new float[]{(float) color.r(), (float) color.g(), (float) color.b()},
                PDDeviceRGB.INSTANCE);
    }

    /**
     * Les polices standard du PDF ne codent que le latin occidental. Un caractere absent
     * ferait echouer l'ecriture entiere : on remplace plutot que de perdre le document.
     */
    private static String sanitise(String value) {
        if (value == null) {
            return "";
        }
        StringBuilder clean = new StringBuilder(value.length());
        for (char c : value.toCharArray()) {
            clean.append(c == '’' ? '\'' : c < 32 && c != '\n' ? ' ' : c > 255 ? '?' : c);
        }
        return clean.toString().replace('\n', ' ');
    }
}
