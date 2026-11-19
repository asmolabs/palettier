use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupDocument {
    #[serde(default = "default_format_version")]
    pub format_version: i32,
    #[serde(default)]
    pub exported_at: Option<String>,
    #[serde(default)]
    pub application: Option<String>,
    #[serde(default)]
    pub paints: Vec<BackupPaint>,
    #[serde(default)]
    pub palettes: Vec<BackupPalette>,
    #[serde(default)]
    pub projects: Vec<BackupProject>,
    #[serde(default)]
    pub recipes: Vec<BackupRecipe>,
    #[serde(default, skip_serializing_if = "IndexMap::is_empty")]
    pub settings: IndexMap<String, String>,
}

fn default_format_version() -> i32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupPaint {
    pub brand: String,
    pub name: String,
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub pigments: Vec<String>,
    pub hex: String,
    #[serde(default)]
    pub tint_hex: Option<String>,
    pub opacity: String,
    pub drying_class: String,
    #[serde(default = "default_tinting")]
    pub tinting_strength: f64,
    #[serde(default)]
    pub owned: bool,
    #[serde(default)]
    pub color_derived: bool,
    #[serde(default)]
    pub notes: String,
    /// "OIL" or "ACRYLIC"; absent from older backups, which held oil only.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub system: String,
    #[serde(default)]
    pub user_added: bool,
}

fn default_tinting() -> f64 {
    0.5
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupPaintRef {
    pub brand: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupPalette {
    pub name: String,
    #[serde(default)]
    pub purpose: String,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub paints: Vec<BackupPaintRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupProject {
    pub name: String,
    #[serde(default)]
    pub subject: String,
    #[serde(default)]
    pub approach: String,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub palette_name: Option<String>,
    /// "OIL" or "ACRYLIC"; absent from older backups, which held oil only.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub system: String,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub paints: Vec<BackupPaintRef>,
    #[serde(default)]
    pub zones: Vec<BackupZone>,
    #[serde(default)]
    pub photos: Vec<BackupPhoto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupZone {
    pub name: String,
    #[serde(default)]
    pub material: String,
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    pub layers: Vec<BackupLayer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupLayer {
    pub role: String,
    pub target_hex: String,
    #[serde(default)]
    pub technique: String,
    #[serde(default)]
    pub note: String,
    #[serde(default = "default_layer_kind")]
    pub kind: String,
    #[serde(default)]
    pub applied_at: Option<String>,
    #[serde(default)]
    pub applied_temperature: Option<f64>,
    #[serde(default)]
    pub applied_humidity: Option<f64>,
    #[serde(default)]
    pub applied_ventilation: Option<String>,
    #[serde(default)]
    pub applied_drying_class: Option<String>,
}

fn default_layer_kind() -> String {
    "LADDER".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupPhoto {
    pub file: String,
    pub role: String,
    #[serde(default)]
    pub caption: String,
    #[serde(default)]
    pub added_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupRecipe {
    pub name: String,
    #[serde(default)]
    pub subject: String,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub steps: Vec<BackupStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupStep {
    pub technique: String,
    #[serde(default)]
    pub paint_mix: String,
    pub medium: String,
    #[serde(default)]
    pub medium_ratio: f64,
    pub thickness: String,
    #[serde(default)]
    pub note: String,
}
