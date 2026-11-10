use crate::engine::{ChatEngine, ImageAttachment};
use palettier_domain::error::{DomainError, DomainResult};
use palettier_domain::paint::{Paint, PaintMatcher};
use palettier_image::ImageDecoder;
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
    decoder: ImageDecoder,
    matcher: PaintMatcher,
}

impl TubeRecognitionService {
    const MAX_DIMENSION: u32 = 1568;

    const SYSTEM_PROMPT: &'static str = r#"Tu examines la photo d'un ou plusieurs tubes de peinture à l'huile posés sur un établi. Pour chaque tube dont l'étiquette est lisible, relève :
- brand    : la marque (ex. "Abteilung 502", "Winsor & Newton", "Scale75", "Sennelier", "Old Holland", "Rembrandt", "Gamblin", "Michael Harding")
- name     : le nom exact de la couleur tel qu'écrit sur le tube, de préférence en anglais s'il figure en plusieurs langues (ex. "Burnt Umber", "Starship Filth", "Basic Flesh Tone")
- code     : la référence ou le numéro sur le tube s'il est visible (ex. "ABT005", "S75-012", "074"), sinon ""
- pigments : les codes pigments C.I. s'ils sont visibles (ex. "PBr7", "PW6, PY42"), sinon ""

Si une étiquette est trop floue ou retournée, mentionne-le dans "notes" plutôt que de deviner."#;

    const USER_PROMPT: &'static str = "Relève tous les tubes visibles sur cette photo.";

    pub fn new(engine: Arc<dyn ChatEngine>) -> Self {
        Self {
            engine,
            decoder: ImageDecoder,
            matcher: PaintMatcher,
        }
    }

    pub async fn recognize(
        &self,
        image_bytes: &[u8],
        catalog: &[Paint],
    ) -> DomainResult<RecognitionResult> {
        let scaled = self.decoder.scale_to(image_bytes, Self::MAX_DIMENSION)?;
        let raw = self
            .engine
            .complete(
                Self::SYSTEM_PROMPT,
                Self::USER_PROMPT,
                &[ImageAttachment::jpeg(scaled)],
                Some(&TUBE_SCHEMA),
            )
            .await?;

        let parsed = Self::parse(&raw)?;
        let tubes = parsed
            .tubes
            .into_iter()
            .filter(|t| !t.name.trim().is_empty() || !t.code.trim().is_empty())
            .map(|t| {
                let label = format!("{} {} {}", t.brand, t.name, t.code);
                let matched = self
                    .matcher
                    .match_label(Some(&label), catalog)
                    .map(|m| m.paint);
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
        let start = raw.find('{');
        let end = raw.rfind('}');
        let cleaned = match (start, end) {
            (Some(s), Some(e)) if e > s => &raw[s..=e],
            _ => raw,
        };
        serde_json::from_str::<TubeList>(cleaned).map_err(|e| {
            let preview: String = raw.chars().take(200).collect();
            DomainError::InvalidArgument(format!(
                "Réponse du modèle illisible ({e}) : {preview}"
            ))
        })
    }
}
