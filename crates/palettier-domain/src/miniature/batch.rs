//! Painting in series: squads, army recipe sheets, mother mixes.

use super::*;

/// Batch painting tracker for an army squad / platoon.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchSquad {
    pub name: String,
    pub miniature_count: usize,
    pub steps: Vec<String>,
    /// Matrix of [miniature_idx][step_idx]
    pub progress: Vec<Vec<bool>>,
}

impl BatchSquad {
    pub fn new(name: impl Into<String>, miniature_count: usize, steps: Vec<String>) -> Self {
        let count = miniature_count.clamp(1, 100);
        let s_len = steps.len();
        let progress = vec![vec![false; s_len]; count];
        Self {
            name: name.into(),
            miniature_count: count,
            steps,
            progress,
        }
    }

    pub fn standard_space_marine_squad(name: &str, count: usize) -> Self {
        Self::new(
            name,
            count,
            vec![
                "1. Sous-couche zénithale".to_string(),
                "2. Base armure".to_string(),
                "3. Pin wash d'huile".to_string(),
                "4. Nettoyage réductif".to_string(),
                "5. Éclaircissement d'arêtes".to_string(),
                "6. Cuirs & Sacoches".to_string(),
                "7. Armes & Métaux".to_string(),
                "8. Lentilles / OSL".to_string(),
                "9. Weathering & Socle".to_string(),
            ],
        )
    }

    pub fn total_tasks(&self) -> usize {
        self.miniature_count * self.steps.len()
    }

    pub fn completed_tasks(&self) -> usize {
        self.progress
            .iter()
            .flat_map(|row| row.iter())
            .filter(|&&c| c)
            .count()
    }

    pub fn progress_pct(&self) -> f64 {
        let total = self.total_tasks();
        if total == 0 {
            return 100.0;
        }
        (self.completed_tasks() as f64 / total as f64) * 100.0
    }

    pub fn toggle(&mut self, miniature_idx: usize, step_idx: usize) {
        if let Some(cell) = self
            .progress
            .get_mut(miniature_idx)
            .and_then(|row| row.get_mut(step_idx))
        {
            *cell = !*cell;
        }
    }

    /// Recommended mother mix volume multiplier (e.g. 10 miniatures require ~2.5x volume of paint).
    pub fn mother_mix_multiplier(&self) -> f64 {
        1.0 + (self.miniature_count as f64 - 1.0) * 0.22
    }
}

/// Printable / exportable army or squad painting recipe card.
///
/// Starts blank: the painter fills it in, the base colour comes from a catalog paint and
/// the oil mix is computed on the palette, never typed in by the application.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ArmyRecipeSheet {
    pub army_name: String,
    pub unit_name: String,
    pub faction: String,
    pub primer: String,
    pub base_acrylic: String,
    pub base_hex: String,
    pub oil_mix_formula: String,
    pub mixed_hex: String,
    pub wash_technique: String,
    pub weathering: String,
    pub varnish: String,
    pub notes: String,
}

impl ArmyRecipeSheet {
    pub fn to_markdown(&self, locale: Locale) -> String {
        let t = SheetTitles::of(locale);
        format!(
            "# {} : {} - {}\n\n\
             **{}** : {}\n\n\
             ---\n\n\
             ### {}\n\
             - {}\n\n\
             ### {}\n\
             - **{}** {} (`{}`)\n\
             - **{}** {} (`{}`)\n\n\
             ### {}\n\
             - {}\n\n\
             ### {}\n\
             - {}\n\n\
             ### {}\n\
             - **{}** : {}\n\n\
             ### {}\n\
             {}\n",
            t.sheet, self.army_name, self.unit_name,
            t.faction, self.faction,
            t.primer, self.primer,
            t.base,
            t.reference, self.base_acrylic, self.base_hex,
            t.mix, self.oil_mix_formula, self.mixed_hex,
            t.wash, self.wash_technique,
            t.weathering, self.weathering,
            t.finish, t.varnish, self.varnish,
            t.notes, self.notes,
        )
    }

    pub fn to_printable_html(&self, locale: Locale) -> String {
        let t = SheetTitles::of(locale);
        format!(
            r#"<!DOCTYPE html>
<html lang="{lang}">
<head>
<meta charset="utf-8">
<title>{army} - {unit}</title>
<style>
  body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; color: #1e293b; background: #ffffff; padding: 32px; max-width: 800px; margin: 0 auto; line-height: 1.5; }}
  h1 {{ font-size: 24px; border-bottom: 2px solid #e2e8f0; padding-bottom: 8px; margin-bottom: 6px; }}
  .subtitle {{ color: #64748b; font-size: 14px; margin-bottom: 24px; }}
  .card {{ border: 1px solid #cbd5e1; border-radius: 8px; padding: 16px; margin-bottom: 16px; page-break-inside: avoid; }}
  .card-title {{ font-size: 14px; font-weight: 700; color: #0f172a; text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 8px; }}
  .swatch {{ display: inline-block; width: 18px; height: 18px; border-radius: 4px; vertical-align: middle; margin-right: 6px; border: 1px solid rgba(0,0,0,0.2); }}
  .pill {{ display: inline-block; padding: 2px 8px; border-radius: 999px; font-size: 11px; font-weight: 600; background: #f1f5f9; color: #475569; }}
  @media print {{ body {{ padding: 0; }} }}
</style>
</head>
<body>
  <h1>{army}</h1>
  <div class="subtitle">{unit} · <strong>{faction}</strong></div>

  <div class="card">
    <div class="card-title">{t_primer}</div>
    <div>{primer}</div>
  </div>

  <div class="card">
    <div class="card-title">{t_base}</div>
    <div style="margin-bottom: 8px;">
      <span class="swatch" style="background-color: {base_hex};"></span>
      <strong>{t_reference}</strong> {base_acrylic} (<code>{base_hex}</code>)
    </div>
    <div>
      <span class="swatch" style="background-color: {mixed_hex};"></span>
      <strong>{t_mix}</strong> {oil_mix_formula} (<code>{mixed_hex}</code>)
    </div>
  </div>

  <div class="card">
    <div class="card-title">{t_wash}</div>
    <div>{wash_technique}</div>
  </div>

  <div class="card">
    <div class="card-title">{t_weathering}</div>
    <div>{weathering}</div>
  </div>

  <div class="card">
    <div class="card-title">{t_finish}</div>
    <div><span class="pill">{t_varnish}</span> {varnish}</div>
  </div>

  <div class="card">
    <div class="card-title">{t_notes}</div>
    <div style="font-style: italic; color: #475569;">{notes}</div>
  </div>
</body>
</html>"#,
            army = self.army_name,
            unit = self.unit_name,
            faction = self.faction,
            primer = self.primer,
            base_acrylic = self.base_acrylic,
            base_hex = self.base_hex,
            oil_mix_formula = self.oil_mix_formula,
            mixed_hex = self.mixed_hex,
            wash_technique = self.wash_technique,
            weathering = self.weathering,
            varnish = self.varnish,
            notes = self.notes,
            lang = t.lang,
            t_primer = t.primer,
            t_base = t.base,
            t_reference = t.reference,
            t_mix = t.mix,
            t_wash = t.wash,
            t_weathering = t.weathering,
            t_finish = t.finish,
            t_varnish = t.varnish,
            t_notes = t.notes,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MiniatureScale {
    Scale28mm,
    Scale32mm,
    Scale54mm,
    Scale75mm,
    Bust1_10,
}

impl MiniatureScale {
    pub const ALL: [MiniatureScale; 5] = [
        MiniatureScale::Scale28mm,
        MiniatureScale::Scale32mm,
        MiniatureScale::Scale54mm,
        MiniatureScale::Scale75mm,
        MiniatureScale::Bust1_10,
    ];

    pub fn label(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::Scale28mm, Locale::Fr) => "28mm (Infanterie classique / Historique)",
            (Self::Scale28mm, Locale::En) => "28mm (Classic infantry / Historical)",
            (Self::Scale32mm, Locale::Fr) => "32mm Héroïque (Space Marines, Warhammer 40k)",
            (Self::Scale32mm, Locale::En) => "32mm Heroic (Space Marines, 40k)",
            (Self::Scale54mm, Locale::Fr) => "54mm (Grand format d'infanterie)",
            (Self::Scale54mm, Locale::En) => "54mm (Large infantry scale)",
            (Self::Scale75mm, Locale::Fr) => "75mm (Figurine d'exposition)",
            (Self::Scale75mm, Locale::En) => "75mm (Display figure)",
            (Self::Bust1_10, Locale::Fr) => "Buste 1/10 ou 1/12 (Grande surface)",
            (Self::Bust1_10, Locale::En) => "1/10 or 1/12 Bust (Large surface)",
        }
    }

    pub fn surface_multiplier(&self) -> f64 {
        match self {
            Self::Scale28mm => 1.0,
            Self::Scale32mm => 1.4,
            Self::Scale54mm => 3.2,
            Self::Scale75mm => 6.0,
            Self::Bust1_10 => 12.0,
        }
    }
}

pub struct BatchMotherMixRequirements {
    pub volume_ml: f64,
    pub matchstick_heads: u32,
    pub hazelnuts: f64,
    pub open_palette_hours: u32,
    pub airtight_pot_days: u32,
    pub freezer_days: u32,
    pub advice: String,
}

pub struct BatchMotherMixCalculator;

impl BatchMotherMixCalculator {
    pub fn calculate(
        scale: MiniatureScale,
        model_count: u32,
        is_full_coverage: bool,
        locale: Locale,
    ) -> BatchMotherMixRequirements {
        let count = model_count.clamp(1, 150) as f64;
        let base_ml_per_model = if is_full_coverage { 0.08 } else { 0.03 };
        let volume_ml = (base_ml_per_model * scale.surface_multiplier() * count).max(0.05);

        let matchstick_heads = (volume_ml / 0.02).ceil() as u32;
        let hazelnuts = (volume_ml / 0.3 * 10.0).round() / 10.0;

        let advice = match locale {
            Locale::Fr => format!(
                "Pour {} figurine(s) à l'échelle {} : préparez environ {} ml de mélange mère (~{} noisettes de pâte). Conservez dans un petit godet hermétique pour éviter l'oxydation entre vos sessions.",
                model_count, scale.label(locale), (volume_ml * 100.0).round() / 100.0, hazelnuts
            ),
            Locale::En => format!(
                "For {} model(s) at {} scale: prepare approx {} ml of mother mix (~{} hazelnut portions). Store in an airtight screw pot between painting sessions.",
                model_count, scale.label(locale), (volume_ml * 100.0).round() / 100.0, hazelnuts
            ),
        };

        BatchMotherMixRequirements {
            volume_ml,
            matchstick_heads,
            hazelnuts,
            open_palette_hours: 12,
            airtight_pot_days: 7,
            freezer_days: 30,
            advice,
        }
    }
}

/// Headings of the printed army recipe sheet, in the reader's language.
struct SheetTitles {
    lang: &'static str,
    sheet: &'static str,
    faction: &'static str,
    primer: &'static str,
    base: &'static str,
    reference: &'static str,
    mix: &'static str,
    wash: &'static str,
    weathering: &'static str,
    finish: &'static str,
    varnish: &'static str,
    notes: &'static str,
}

impl SheetTitles {
    fn of(locale: Locale) -> Self {
        Self {
            lang: locale.tr("fr", "en"),
            sheet: locale.tr("Fiche de recette", "Recipe sheet"),
            faction: locale.tr("Faction / système", "Faction / system"),
            primer: locale.tr("1. Sous-couche & préparation", "1. Primer & preparation"),
            base: locale.tr("2. Couleur de base & formule d'huile", "2. Base colour & oil formula"),
            reference: locale.tr("Acrylique de référence :", "Reference acrylic:"),
            mix: locale.tr("Mélange d'huiles :", "Oil mix:"),
            wash: locale.tr("3. Lavis & travail réductif", "3. Washes & reductive work"),
            weathering: locale.tr("4. Weathering & altération", "4. Weathering"),
            finish: locale.tr("5. Finition & protection", "5. Finish & protection"),
            varnish: locale.tr("Vernis", "Varnish"),
            notes: locale.tr("Notes d'atelier", "Studio notes"),
        }
    }
}
