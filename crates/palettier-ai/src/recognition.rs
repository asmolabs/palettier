use crate::engine::{ChatEngine, ImageAttachment};
use crate::schema::parse_json_object;
use palettier_domain::error::DomainResult;
use palettier_domain::i18n::Locale;
use palettier_domain::paint::{Paint, PaintSystem, match_paint_label};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::{Arc, LazyLock};

#[derive(Debug, Clone, PartialEq)]
pub struct RecognizedTube {
    pub read_brand: String,
    pub read_name: String,
    pub read_code: String,
    pub read_pigments: String,
    pub matched: Option<Paint>,
}

impl RecognizedTube {
    pub fn is_in_catalog(&self) -> bool {
        self.matched.is_some()
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RecognitionResult {
    pub tubes: Vec<RecognizedTube>,
    pub notes: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TubeDraft {
    #[serde(default)]
    pub brand: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub pigments: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TubeList {
    #[serde(default)]
    pub tubes: Vec<TubeDraft>,
    #[serde(default)]
    pub notes: String,
}

pub static TUBE_SCHEMA: LazyLock<Value> = LazyLock::new(|| {
    json!({
        "type": "object",
        "properties": {
            "tubes": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "brand": { "type": "string" },
                        "name": { "type": "string" },
                        "code": { "type": "string" },
                        "pigments": { "type": "string" }
                    },
                    "required": ["brand", "name", "code", "pigments"]
                }
            },
            "notes": { "type": "string" }
        },
        "required": ["tubes", "notes"]
    })
});

pub struct TubeRecognitionService {
    engine: Arc<dyn ChatEngine>,
}

impl TubeRecognitionService {
    const MAX_DIMENSION: u32 = 1568;

    const SYSTEM_PROMPT_OIL_FR: &'static str = r#"Tu examines la photo d'un ou plusieurs tubes de peinture à l'huile posés sur un établi. Pour chaque tube dont l'étiquette est lisible, relève :
- brand    : la marque (ex. "Abteilung 502", "Winsor & Newton", "Scale75", "Sennelier", "Old Holland", "Rembrandt", "Gamblin", "Michael Harding")
- name     : le nom exact de la couleur tel qu'écrit sur le tube, de préférence en anglais s'il figure en plusieurs langues (ex. "Burnt Umber", "Starship Filth", "Basic Flesh Tone")
- code     : la référence ou le numéro sur le tube s'il est visible (ex. "ABT005", "S75-012", "074"), sinon ""
- pigments : les codes pigments C.I. s'ils sont visibles (ex. "PBr7", "PW6, PY42"), sinon ""

Si une étiquette est trop floue ou retournée, mentionne-le dans "notes" plutôt que de deviner."#;

    const SYSTEM_PROMPT_OIL_EN: &'static str = r#"You are inspecting a photo of one or more oil paint tubes on a workbench. For every tube with a readable label, extract:
- brand    : the manufacturer (e.g. "Abteilung 502", "Winsor & Newton", "Scale75", "Sennelier", "Old Holland", "Rembrandt", "Gamblin", "Michael Harding")
- name     : the exact colour name printed on the tube, preferring English when multilingual (e.g. "Burnt Umber", "Starship Filth", "Basic Flesh Tone")
- code     : the product code or number if visible (e.g. "ABT005", "S75-012", "074"), otherwise ""
- pigments : the C.I. pigment codes if visible (e.g. "PBr7", "PW6, PY42"), otherwise ""

If a label is too blurry or turned away, mention it in "notes" rather than guessing."#;

    const SYSTEM_PROMPT_ACRYLIC_FR: &'static str = r#"Tu examines la photo d'un ou plusieurs pots ou flacons de peinture acrylique pour figurines posés sur un établi. Pour chaque flacon ou pot dont l'étiquette est lisible, relève :
- brand    : la marque (ex. "Citadel", "Vallejo Model Color", "Vallejo Game Color", "Army Painter Warpaints", "AK Interactive 3rd Gen", "Pro Acryl", "Scale75", "Kimera Kolors")
- name     : le nom exact de la couleur tel qu'écrit sur l'étiquette (ex. "Mephiston Red", "Ice Yellow", "Dark Sea Blue")
- code     : la référence ou le numéro si visible (ex. "70.951", "AK11001"), sinon ""
- pigments : les codes pigments C.I. s'ils sont visibles (ex. "PR112", "PW6, PY42"), sinon ""

Si une étiquette est trop floue ou retournée, mentionne-le dans "notes" plutôt que de deviner."#;

    const SYSTEM_PROMPT_ACRYLIC_EN: &'static str = r#"You are inspecting a photo of one or more miniature acrylic paint pots or dropper bottles on a workbench. For every bottle or pot with a readable label, extract:
- brand    : the range or manufacturer (e.g. "Citadel", "Vallejo Model Color", "Vallejo Game Color", "Army Painter Warpaints", "AK Interactive 3rd Gen", "Pro Acryl", "Scale75", "Kimera Kolors")
- name     : the exact colour name printed on the label (e.g. "Mephiston Red", "Ice Yellow", "Dark Sea Blue")
- code     : the product code or number if visible (e.g. "70.951", "AK11001"), otherwise ""
- pigments : the C.I. pigment codes if visible (e.g. "PR112", "PW6, PY42"), otherwise ""

If a label is too blurry or turned away, mention it in "notes" rather than guessing."#;

    const USER_PROMPT_FR: &'static str = "Relève tous les tubes ou flacons visibles sur cette photo.";
    const USER_PROMPT_EN: &'static str = "List all paint tubes or bottles visible in this photo.";

    pub fn new(engine: Arc<dyn ChatEngine>) -> Self {
        Self { engine }
    }

    pub async fn recognize(
        &self,
        image_bytes: &[u8],
        catalog: &[Paint],
    ) -> DomainResult<RecognitionResult> {
        self.recognize_in(image_bytes, catalog, PaintSystem::Oil, Locale::Fr)
            .await
    }

    pub async fn recognize_in(
        &self,
        image_bytes: &[u8],
        catalog: &[Paint],
        system: PaintSystem,
        locale: Locale,
    ) -> DomainResult<RecognitionResult> {
        let scaled = palettier_image::scale_to_encoded(image_bytes, Self::MAX_DIMENSION)?;
        let (system_prompt, user_prompt) = match (system, locale) {
            (PaintSystem::Oil, Locale::Fr) => (Self::SYSTEM_PROMPT_OIL_FR, Self::USER_PROMPT_FR),
            (PaintSystem::Oil, Locale::En) => (Self::SYSTEM_PROMPT_OIL_EN, Self::USER_PROMPT_EN),
            (PaintSystem::Acrylic, Locale::Fr) => {
                (Self::SYSTEM_PROMPT_ACRYLIC_FR, Self::USER_PROMPT_FR)
            }
            (PaintSystem::Acrylic, Locale::En) => {
                (Self::SYSTEM_PROMPT_ACRYLIC_EN, Self::USER_PROMPT_EN)
            }
        };
        let raw = self
            .engine
            .complete(
                system_prompt,
                user_prompt,
                &[ImageAttachment::new(scaled.bytes, scaled.mime_type)],
                Some(&TUBE_SCHEMA),
            )
            .await?;

        let parsed = Self::parse_in(&raw, locale)?;
        let system_catalog: Vec<Paint> =
            catalog.iter().filter(|p| p.system == system).cloned().collect();
        let match_pool = if system_catalog.is_empty() {
            catalog
        } else {
            &system_catalog
        };

        let tubes = parsed
            .tubes
            .into_iter()
            .filter(|t| !t.name.trim().is_empty() || !t.code.trim().is_empty())
            .map(|t| {
                let label = format!("{} {} {}", t.brand, t.name, t.code);
                let matched = match_paint_label(Some(&label), match_pool).map(|m| m.paint);
                RecognizedTube {
                    read_brand: t.brand,
                    read_name: t.name,
                    read_code: t.code,
                    read_pigments: t.pigments,
                    matched,
                }
            })
            .collect();

        Ok(RecognitionResult {
            tubes,
            notes: parsed.notes,
        })
    }

    pub fn parse(raw: &str) -> DomainResult<TubeList> {
        Self::parse_in(raw, Locale::Fr)
    }

    pub fn parse_in(raw: &str, locale: Locale) -> DomainResult<TubeList> {
        parse_json_object(raw, locale)
    }
}
