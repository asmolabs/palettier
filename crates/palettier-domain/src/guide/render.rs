use super::MixingGuide;
use crate::html::{escape_html, sanitize_hex_css, sanitize_image_data_uri};
use crate::i18n::Locale;

impl MixingGuide {
    /// Generates a self-contained A4 printable workshop card in HTML/CSS in the chosen locale.
    pub fn to_printable_html(&self, photo_data_uri: Option<&str>, locale: Locale) -> String {
        let mut html = String::with_capacity(16384);
        html.push_str("<!DOCTYPE html>\n<html lang=\"");
        html.push_str(locale.code());
        html.push_str(
            r#"">
<head>
<meta charset="utf-8">
<title>"#,
        );
        html.push_str(&escape_html(&self.title));
        html.push_str(" — ");
        html.push_str(&escape_html(&self.subtitle));
        html.push_str(
            r#"</title>
<style>
  @page { size: A4 portrait; margin: 16mm 16mm 18mm 16mm; }
  * { box-sizing: border-box; -webkit-print-color-adjust: exact !important; print-color-adjust: exact !important; }
  body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    color: #262320;
    background: #FFFFFF;
    margin: 0;
    padding: 24px 32px;
    font-size: 10.5pt;
    line-height: 1.42;
    max-width: 210mm;
    margin-left: auto;
    margin-right: auto;
  }
  .toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: #FAF6F0;
    border: 1px solid #E5DEC9;
    padding: 10px 16px;
    border-radius: 8px;
    margin-bottom: 22px;
  }
  .toolbar button {
    background: #7C2D12;
    color: #FFFFFF;
    border: none;
    border-radius: 6px;
    padding: 8px 16px;
    font-weight: 600;
    cursor: pointer;
    font-size: 10pt;
  }
  @media print {
    body { padding: 0; max-width: none; }
    .toolbar { display: none !important; }
  }
  .header-grid {
    display: flex;
    gap: 24px;
    align-items: flex-start;
    margin-bottom: 18px;
  }
  .piece-photo {
    width: 175px;
    max-height: 225px;
    object-fit: cover;
    border-radius: 3px;
    border: 1px solid #D6D0C4;
    flex-shrink: 0;
  }
  .header-main { flex: 1; }
  h1 {
    font-size: 22pt;
    font-weight: 800;
    margin: 0 0 4px 0;
    color: #1F1C18;
    letter-spacing: -0.02em;
  }
  .subtitle {
    font-size: 11.5pt;
    color: #57524A;
    margin: 0 0 10px 0;
  }
  .disclaimer {
    font-size: 9pt;
    color: #6E675F;
    margin: 0 0 14px 0;
  }
  h2 {
    font-size: 13pt;
    font-weight: 700;
    color: #7C2D12;
    margin: 18px 0 8px 0;
    page-break-after: avoid;
  }
  table.guide-table {
    width: 100%;
    border-collapse: collapse;
    margin-bottom: 10px;
  }
  table.guide-table tr {
    border-bottom: 1px solid #EBE6DC;
    page-break-inside: avoid;
  }
  table.guide-table td {
    padding: 5px 8px 5px 0;
    vertical-align: middle;
  }
  .swatch-cell {
    width: 44px;
    padding-right: 10px !important;
  }
  .swatch-box {
    width: 36px;
    height: 22px;
    border: 1px solid #B8B0A4;
    border-radius: 2px;
  }
  .code-cell {
    width: 42px;
    font-weight: 700;
    color: #1F1C18;
  }
  .role-cell {
    width: 135px;
    font-weight: 700;
    color: #1F1C18;
  }
  .behavior-cell {
    color: #57524A;
    font-size: 9.5pt;
    width: 36%;
  }
  .note-paragraph {
    font-size: 9pt;
    color: #4A453E;
    margin: 6px 0 14px 0;
    line-height: 1.45;
  }
  ol.advice-list {
    margin: 6px 0 0 0;
    padding-left: 20px;
  }
  ol.advice-list li {
    margin-bottom: 6px;
    page-break-inside: avoid;
  }
  .footer {
    margin-top: 24px;
    padding-top: 8px;
    border-top: 1px solid #EBE6DC;
    text-align: right;
    font-size: 8.5pt;
    color: #8C8479;
  }
</style>
</head>
<body>
<div class="toolbar">
  <div><strong>"#,
        );
        html.push_str(locale.tr("Fiche d'établi Palettier", "Palettier Workbench Sheet"));
        html.push_str("</strong> — ");
        html.push_str(locale.tr(
            "Prête à imprimer en A4 ou enregistrer en PDF",
            "Ready to print on A4 or save as PDF",
        ));
        html.push_str(
            r#"</div>
  <button onclick="window.print()">"#,
        );
        html.push_str(locale.tr(
            "Imprimer / Enregistrer en PDF (⌘P)",
            "Print / Save to PDF (⌘P)",
        ));
        html.push_str(
            r#"</button>
</div>
<div class="header-grid">
"#,
        );

        if let Some(uri) = photo_data_uri.and_then(sanitize_image_data_uri) {
            let alt_txt = escape_html(locale.tr("Pièce de référence", "Reference Piece"));
            html.push_str(&format!(
                r#"  <img class="piece-photo" src="{uri}" alt="{alt_txt}"/>"#
            ));
        }

        html.push_str(r#"  <div class="header-main">"#);
        html.push_str(&format!(
            "<h1>{}</h1>\n<p class=\"subtitle\">{}</p>\n<p class=\"disclaimer\">{}</p>\n",
            escape_html(&self.title),
            escape_html(&self.subtitle),
            escape_html(&self.disclaimer)
        ));

        // Section Ta palette / Your palette
        if !self.palette_entries.is_empty() {
            html.push_str(&format!(
                "<h2>{}</h2>\n<table class=\"guide-table\">\n",
                locale.tr("Ta palette", "Your Palette")
            ));
            for entry in &self.palette_entries {
                let code_suffix = if entry.paint.code.trim().is_empty() {
                    String::new()
                } else {
                    format!(
                        " <span style=\"color:#78716A;\">({})</span>",
                        escape_html(&entry.paint.code)
                    )
                };
                html.push_str(&format!(
                    "<tr><td class=\"swatch-cell\"><div class=\"swatch-box\" style=\"background:{};\"></div></td><td class=\"code-cell\">{}</td><td>{}{code_suffix}</td><td class=\"behavior-cell\">{}</td></tr>\n",
                    sanitize_hex_css(&entry.paint.hex_color),
                    escape_html(&entry.short_code),
                    escape_html(&entry.paint.name),
                    escape_html(&entry.behavior),
                ));
            }
            html.push_str("</table>\n");
        }
        html.push_str("  </div>\n</div>\n");

        // Mélanges intermédiaires / Mother mixes
        if !self.mother_mixes.is_empty() {
            html.push_str(&format!(
                "<h2>{}</h2>\n<table class=\"guide-table\">\n",
                locale.tr(
                    "Mélanges intermédiaires à préparer en début de séance",
                    "Intermediate Mother Mixes to Prepare Before Painting"
                )
            ));
            for mm in &self.mother_mixes {
                html.push_str(&format!(
                    "<tr><td class=\"swatch-cell\"><div class=\"swatch-box\" style=\"background:{};\"></div></td><td class=\"role-cell\"><strong>{}</strong> — {}</td><td>{}</td></tr>\n",
                    sanitize_hex_css(&mm.hex_color),
                    escape_html(&mm.code),
                    escape_html(&mm.name),
                    escape_html(&mm.recipe_text),
                ));
            }
            html.push_str("</table>\n");
            html.push_str(&format!(
                "<p class=\"note-paragraph\">{}</p>\n",
                escape_html(&self.mother_mixes_note)
            ));
        }

        // Lavis sur sous-couche blanche / Washes
        if !self.washes.is_empty() {
            html.push_str(&format!(
                "<h2>{}</h2>\n<table class=\"guide-table\">\n",
                locale.tr(
                    "Lavis sur sous-couche blanche (à l'huile)",
                    "Oil Washes Over White Undercoat"
                )
            ));
            for w in &self.washes {
                html.push_str(&format!(
                    "<tr><td class=\"swatch-cell\"><div class=\"swatch-box\" style=\"background:{};\"></div></td><td class=\"role-cell\">{}</td><td>{}</td></tr>\n",
                    sanitize_hex_css(&w.hex_color),
                    escape_html(&w.name),
                    escape_html(&w.recipe_text),
                ));
            }
            html.push_str("</table>\n");
            html.push_str(&format!(
                "<p class=\"note-paragraph\"><strong>{}</strong> {}</p>\n",
                locale.tr("Méthode :", "Method:"),
                escape_html(&self.washes_method)
            ));
        }

        // Zones
        for zone in &self.zones {
            let zone_heading = if zone.material.trim().is_empty()
                || zone
                    .name
                    .to_lowercase()
                    .contains(&zone.material.to_lowercase())
            {
                zone.name.clone()
            } else {
                format!("{} ({})", zone.name, zone.material)
            };
            html.push_str(&format!(
                "<h2>{}</h2>\n<table class=\"guide-table\">\n",
                escape_html(&zone_heading)
            ));
            for row in &zone.rows {
                html.push_str(&format!(
                    "<tr><td class=\"swatch-cell\"><div class=\"swatch-box\" style=\"background:{};\"></div></td><td class=\"role-cell\">{}</td><td>{}</td></tr>\n",
                    sanitize_hex_css(&row.swatch_hex),
                    escape_html(&row.role),
                    escape_html(&row.painter_recipe),
                ));
            }
            html.push_str("</table>\n");
            if !zone.note.trim().is_empty() {
                html.push_str(&format!(
                    "<p class=\"note-paragraph\">{}</p>\n",
                    escape_html(&zone.note)
                ));
            }
        }

        // Conseils / Studio tips
        if !self.advice.is_empty() {
            html.push_str(&format!(
                "<h2>{}</h2>\n<ol class=\"advice-list\">\n",
                locale.tr("Conseils", "Studio Tips")
            ));
            for tip in &self.advice {
                html.push_str(&format!("  <li>{}</li>\n", escape_html(tip)));
            }
            html.push_str("</ol>\n");
        }

        html.push_str(&format!(
            "<div class=\"footer\">{} — {}</div>\n</body>\n</html>",
            escape_html(&self.title),
            escape_html(&self.subtitle)
        ));
        html
    }
}
