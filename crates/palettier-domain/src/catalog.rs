use crate::color::{Colorant, Rgb, linear_to_srgb, mix, srgb_to_linear};
use crate::paint::{DryingClass, Opacity, Paint, Pigment};
use indexmap::{IndexMap, IndexSet};
use std::str::FromStr;

mod generated {
    include!(concat!(env!("OUT_DIR"), "/catalog_files.rs"));
}

pub use generated::{BRAND_FILES, PIGMENTS_CSV};

/// Semicolon-separated CSV reader.
pub struct Csv;

#[derive(Debug, Clone)]
pub struct CsvRow {
    values: IndexMap<String, String>,
    origin: String,
    line_number: usize,
}

impl CsvRow {
    pub fn get(&self, column: &str) -> &str {
        self.values
            .get(column)
            .map(String::as_str)
            .unwrap_or_else(|| panic!("Colonne '{}' absente dans {}", column, self.origin))
    }

    pub fn get_or_default<'a>(&'a self, column: &str, fallback: &'a str) -> &'a str {
        match self.values.get(column) {
            Some(v) if !v.trim().is_empty() => v.as_str(),
            _ => fallback,
        }
    }

    pub fn location(&self) -> String {
        format!("{}:{}", self.origin, self.line_number)
    }
}

impl Csv {
    const SEPARATOR: char = ';';
    const COMMENT: &'static str = "#";

    pub fn read(text: &str, origin: &str) -> Vec<CsvRow> {
        let mut rows = Vec::new();
        let mut headers: Option<Vec<String>> = None;

        for (index, line) in text.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with(Self::COMMENT) {
                continue;
            }
            let cells: Vec<String> = line
                .split(Self::SEPARATOR)
                .map(|c| c.trim().to_string())
                .collect();
            let Some(cols) = &headers else {
                headers = Some(cells);
                continue;
            };
            let mut values = IndexMap::with_capacity(cols.len());
            for (i, col_name) in cols.iter().enumerate() {
                let cell = cells.get(i).cloned().unwrap_or_default();
                values.insert(col_name.clone(), cell);
            }
            rows.push(CsvRow {
                values,
                origin: origin.to_string(),
                line_number: index + 1,
            });
        }
        rows
    }
}

/// Pigment lookup table built from `catalog/pigments.csv`.
#[derive(Debug, Clone)]
pub struct PigmentIndex {
    by_code: IndexMap<String, Pigment>,
}

impl Default for PigmentIndex {
    fn default() -> Self {
        Self::new(PIGMENTS_CSV)
    }
}

impl PigmentIndex {
    pub fn new(pigments_csv: &str) -> Self {
        let mut by_code = IndexMap::new();
        for row in Csv::read(pigments_csv, "pigments.csv") {
            let code = row.get("code").to_string();
            let name = row.get("name").to_string();
            let color = Rgb::of_hex(row.get("hex")).expect("hex pigment valide");
            let opacity = Opacity::from_str(row.get("opacity")).expect("opacité pigment valide");
            let drying_class =
                DryingClass::from_str(row.get("drying")).expect("drying pigment valide");
            let tinting_strength: f64 = row
                .get("tinting")
                .parse()
                .expect("tinting_strength pigment valide");
            let tint_raw = row.get_or_default("tint", "");
            let tint = if tint_raw.is_empty() {
                None
            } else {
                Some(Rgb::of_hex(tint_raw).expect("tint hex pigment valide"))
            };
            let pigment = Pigment {
                code: code.clone(),
                name,
                color,
                opacity,
                drying_class,
                tinting_strength,
                tint,
            };
            by_code.insert(Self::normalise(&code), pigment);
        }
        Self { by_code }
    }

    pub fn find(&self, code: &str) -> Option<&Pigment> {
        self.by_code.get(&Self::normalise(code))
    }

    pub fn all(&self) -> impl Iterator<Item = &Pigment> {
        self.by_code.values()
    }

    pub fn unknown_among<'a>(&self, codes: impl IntoIterator<Item = &'a str>) -> Vec<String> {
        let mut unknown: Vec<String> = codes
            .into_iter()
            .filter(|c| self.find(c).is_none())
            .map(ToString::to_string)
            .collect();
        unknown.sort();
        unknown.dedup();
        unknown
    }

    fn resolve(&self, codes: &IndexSet<String>) -> Vec<Pigment> {
        codes
            .iter()
            .map(|c| {
                self.find(c)
                    .cloned()
                    .unwrap_or_else(|| Pigment::unknown(c))
            })
            .collect()
    }

    pub fn drying_class_for(&self, codes: &IndexSet<String>) -> DryingClass {
        self.resolve(codes)
            .into_iter()
            .fold(DryingClass::Fast, |slowest, p| slowest.slowest(p.drying_class))
    }

    pub fn tinting_strength_for(&self, codes: &IndexSet<String>) -> f64 {
        self.resolve(codes)
            .into_iter()
            .map(|p| p.tinting_strength)
            .reduce(f64::max)
            .unwrap_or(0.5)
    }

    pub fn opacity_for(&self, codes: &IndexSet<String>) -> Opacity {
        self.resolve(codes)
            .into_iter()
            .fold(Opacity::Transparent, |most, p| {
                if most >= p.opacity { most } else { p.opacity }
            })
    }

    pub fn color_for(&self, codes: &IndexSet<String>) -> Rgb {
        let pigments = self.resolve(codes);
        if pigments.is_empty() {
            return Rgb::new(0.5, 0.5, 0.5);
        }
        let colors: Vec<Rgb> = pigments.iter().map(|p| p.color).collect();
        let weights: Vec<f64> = pigments.iter().map(|p| p.tinting_strength).collect();
        mix(&colors, &weights).unwrap_or_else(|_| Rgb::new(0.5, 0.5, 0.5))
    }

    pub fn tint_for(&self, codes: &IndexSet<String>, masstone: Rgb) -> Option<Rgb> {
        let pigments = self.resolve(codes);
        if pigments.iter().all(|p| p.tint.is_none()) {
            return None;
        }

        let mut gain = [0.0; 3];
        let mut total = 0.0;
        for pigment in &pigments {
            let weight = pigment.tinting_strength;
            let pigment_gain = Self::divergence_of(pigment);
            for channel in 0..3 {
                gain[channel] += weight * pigment_gain[channel];
            }
            total += weight;
        }

        let predicted =
            Colorant::of_masstone(masstone).tint(Colorant::REFERENCE_TINT_CONCENTRATION);
        let predicted_linear = [
            srgb_to_linear(predicted.r()),
            srgb_to_linear(predicted.g()),
            srgb_to_linear(predicted.b()),
        ];

        Some(Rgb::new(
            linear_to_srgb(predicted_linear[0] * gain[0] / total),
            linear_to_srgb(predicted_linear[1] * gain[1] / total),
            linear_to_srgb(predicted_linear[2] * gain[2] / total),
        ))
    }

    fn divergence_of(pigment: &Pigment) -> [f64; 3] {
        let Some(observed) = pigment.tint else {
            return [1.0, 1.0, 1.0];
        };
        let predicted =
            Colorant::of_masstone(pigment.color).tint(Colorant::REFERENCE_TINT_CONCENTRATION);
        [
            Self::ratio(observed.r(), predicted.r()),
            Self::ratio(observed.g(), predicted.g()),
            Self::ratio(observed.b(), predicted.b()),
        ]
    }

    fn ratio(observed: f64, predicted: f64) -> f64 {
        srgb_to_linear(observed) / srgb_to_linear(predicted).max(1e-5)
    }

    fn normalise(code: &str) -> String {
        code.trim().to_uppercase().replace(' ', "")
    }
}

/// A brand range file: its filename and CSV content.
#[derive(Debug, Clone)]
pub struct BrandFile {
    pub name: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct LoadedCatalog {
    pub paints: Vec<Paint>,
    pub unknown_pigments: Vec<String>,
}

/// Builds the paint catalog from brand CSV files.
#[derive(Debug, Clone)]
pub struct CatalogLoader {
    pigments: PigmentIndex,
    brands: Vec<BrandFile>,
}

impl Default for CatalogLoader {
    fn default() -> Self {
        Self::new(
            PigmentIndex::default(),
            BRAND_FILES
                .iter()
                .map(|(name, content)| BrandFile {
                    name: (*name).to_string(),
                    content: (*content).to_string(),
                })
                .collect(),
        )
    }
}

impl CatalogLoader {
    pub fn new(pigments: PigmentIndex, brands: Vec<BrandFile>) -> Self {
        Self { pigments, brands }
    }

    pub fn load(&self) -> Vec<Paint> {
        self.load_detailed().paints
    }

    pub fn load_detailed(&self) -> LoadedCatalog {
        let mut paints = Vec::new();
        let mut cited = IndexSet::new();

        for brand in &self.brands {
            for row in Csv::read(&brand.content, &brand.name) {
                let paint = self.to_paint(&row);
                for code in &paint.pigments {
                    cited.insert(code.clone());
                }
                paints.push(paint);
            }
        }
        let unknown = self
            .pigments
            .unknown_among(cited.iter().map(String::as_str));
        LoadedCatalog {
            paints,
            unknown_pigments: unknown,
        }
    }

    fn to_paint(&self, row: &CsvRow) -> Paint {
        let pigment_codes = Self::parse_pigments(row.get("pigments"));
        assert!(
            !pigment_codes.is_empty(),
            "Aucun pigment déclaré en {}",
            row.location()
        );

        let declared_hex = row.get_or_default("hex", "");
        let derived = declared_hex.is_empty();
        let hex = if derived {
            self.pigments.color_for(&pigment_codes).to_hex()
        } else {
            declared_hex.to_string()
        };

        let opacity_raw = row.get_or_default("opacity", "");
        let opacity = if opacity_raw.is_empty() {
            self.pigments.opacity_for(&pigment_codes)
        } else {
            Opacity::from_str(opacity_raw).expect("opacité valide dans fichier de gamme")
        };

        let tint_raw = row.get_or_default("tint", "");
        let tint = if !tint_raw.is_empty() {
            Some(tint_raw.to_string())
        } else {
            let masstone = Rgb::of_hex(&hex).expect("hex valide");
            self.pigments
                .tint_for(&pigment_codes, masstone)
                .map(|c| c.to_hex())
        };

        let verified_str = row.get_or_default("verified", "false");
        let pigments_verified = verified_str.eq_ignore_ascii_case("true");

        Paint {
            id: None,
            brand: row.get("brand").to_string(),
            name: row.get("name").to_string(),
            code: row.get_or_default("code", "").to_string(),
            legacy_code: row.get_or_default("legacy", "").to_string(),
            pigments: pigment_codes.clone(),
            hex_color: hex,
            tint_hex: tint,
            opacity,
            drying_class: self.pigments.drying_class_for(&pigment_codes),
            tinting_strength: self.pigments.tinting_strength_for(&pigment_codes),
            color_derived: derived,
            pigments_verified,
            user_added: false,
            in_stock: true,
            notes: String::new(),
        }
    }

    fn parse_pigments(cell: &str) -> IndexSet<String> {
        cell.split([' ', '\t'])
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(ToString::to_string)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eight_brands_yield_same_680_tubes_as_java() {
        let loaded = CatalogLoader::default().load_detailed();
        assert_eq!(680, loaded.paints.len());

        let mut seen = IndexSet::new();
        let mut duplicates = 0;
        for p in &loaded.paints {
            if !seen.insert((p.brand.clone(), p.name.clone())) {
                duplicates += 1;
            }
        }
        assert_eq!(0, duplicates, "aucun doublon marque+nom");
    }

    #[test]
    fn no_cited_pigment_is_missing_from_table() {
        let loaded = CatalogLoader::default().load_detailed();
        assert!(loaded.unknown_pigments.is_empty());
    }

    #[test]
    fn declared_entry_is_kept_as_is() {
        let loaded = CatalogLoader::default().load_detailed();
        let umber = loaded
            .paints
            .iter()
            .find(|p| p.brand == "Winsor & Newton" && p.name == "Burnt Umber")
            .unwrap();
        assert_eq!(
            IndexSet::from(["PBr7".to_string()]),
            umber.pigments
        );
        assert_eq!(DryingClass::Fast, umber.drying_class);
        assert!(umber.pigments_verified);
        assert!(!umber.color_derived);
    }

    #[test]
    fn entry_without_hex_derives_color_from_pigments() {
        let loaded = CatalogLoader::default().load_detailed();
        let derived: Vec<&Paint> = loaded.paints.iter().filter(|p| p.color_derived).collect();
        assert!(!derived.is_empty());
        for paint in derived {
            assert!(paint.hex_color.starts_with('#'));
        }
    }

    #[test]
    fn tint_is_populated_where_pigments_know_it() {
        let loaded = CatalogLoader::default().load_detailed();
        let with_tint = loaded.paints.iter().filter(|p| p.tint_hex.is_some()).count();
        assert!(with_tint > 400, "seulement {} tubes à deux constantes", with_tint);
    }

    #[test]
    fn slowest_pigment_governs_tube_drying() {
        let index = PigmentIndex::default();
        assert_eq!(
            DryingClass::Fast,
            index.drying_class_for(&IndexSet::from(["PBr7".to_string()]))
        );
        assert_eq!(
            DryingClass::VerySlow,
            index.drying_class_for(&IndexSet::from([
                "PBr7".to_string(),
                "PY35".to_string()
            ]))
        );
    }

    #[test]
    fn unknown_pigment_code_does_not_crash_loader() {
        let index = PigmentIndex::default();
        assert!(index.find("PX999").is_none());
        assert_eq!(
            DryingClass::Medium,
            index.drying_class_for(&IndexSet::from(["PX999".to_string()]))
        );
    }

    #[test]
    fn most_opaque_pigment_governs_default_opacity() {
        let index = PigmentIndex::default();
        let white = index.find("PW6").expect("PW6 présent");
        assert_eq!(Opacity::Opaque, white.opacity);
        assert_eq!(
            Opacity::Opaque,
            index.opacity_for(&IndexSet::from([
                "PW6".to_string(),
                "PB27".to_string()
            ]))
        );
    }
}
