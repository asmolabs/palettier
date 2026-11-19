#![forbid(unsafe_code)]

//! Multimodal AI assistant integrations for Palettier.
//!
//! Provides vision and language model features:
//! - **Structured Plan Generation**: Analyzes miniature reference photos to identify painted zones,
//!   suggests techniques, and recommends paint mixes according to the selected paint system.
//! - **Paint Tube Shelf Recognition**: Identifies paint tubes, brands, and colors from workshop photos.
//! - **Multi-Provider Engine**: Supports Google Gemini, OpenAI, and local Ollama instances.
//! - **Strict JSON Schema**: Guarantees typed structured responses compatible with provider-specific APIs.
//! - **Credential Security**: Redacts and securely handles API keys using `secrecy` and `zeroize`.
//!
//! # Example
//! ```
//! use palettier_ai::strict_schema;
//! use serde_json::json;
//!
//! let raw = json!({
//!     "type": "object",
//!     "properties": { "name": { "type": "string" } }
//! });
//! let strict = strict_schema(&raw);
//! assert_eq!(strict["additionalProperties"], false);
//! ```

/// LLM engine interfaces and provider implementations (Gemini, OpenAI, Ollama).
pub mod engine;

/// Plan generation from miniature reference images and recipe enrichment.
pub mod plan;

/// Vision-based tube recognition from shelf or palette photos.
pub mod recognition;

/// JSON schema transformation utilities for strict structured outputs.
pub mod schema;

/// AI provider settings and secure secret persistence.
pub mod settings;

pub use engine::{
    ChatEngine, ChatEngines, GeminiEngine, ImageAttachment, ModelCatalog, ModelOption,
    OllamaEngine, OpenAiEngine,
};
pub use plan::{
    LayerDraft, PLAN_SCHEMA, PaintingPlanService, PlanDraft, PlanEnricher, SampledZone, ZoneDraft,
};
pub use recognition::{
    RecognitionResult, RecognizedTube, TUBE_SCHEMA, TubeDraft, TubeList, TubeRecognitionService,
};
pub use schema::{open_api_schema, strict_schema};
pub use settings::{AiProvider, AiSettings, AiSettingsStore};

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use indexmap::IndexSet;
    use palettier_domain::color::Rgb;
    use palettier_domain::error::DomainResult;
    use palettier_domain::paint::{DryingClass, Opacity, Paint};
    use palettier_domain::port::SettingsRepository;
    use serde_json::{Value, json};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    struct StubEngine {
        response: String,
    }

    #[async_trait]
    impl ChatEngine for StubEngine {
        async fn complete(
            &self,
            _system: &str,
            _user: &str,
            _images: &[ImageAttachment],
            _schema: Option<&Value>,
        ) -> DomainResult<String> {
            Ok(self.response.clone())
        }
    }

    #[derive(Default)]
    struct MemSettings {
        map: Mutex<HashMap<String, String>>,
    }

    #[async_trait]
    impl SettingsRepository for MemSettings {
        async fn get(&self, key: &str) -> DomainResult<Option<String>> {
            Ok(self.map.lock().unwrap().get(key).cloned())
        }
        async fn put(&self, key: &str, value: &str) -> DomainResult<()> {
            self.map
                .lock()
                .unwrap()
                .insert(key.to_string(), value.to_string());
            Ok(())
        }
        async fn remove(&self, key: &str) -> DomainResult<()> {
            self.map.lock().unwrap().remove(key);
            Ok(())
        }
    }

    fn zorn_palette() -> Vec<Paint> {
        vec![
            Paint {
                id: Some(1),
                brand: "Winsor & Newton".to_string(),
                name: "Titanium White".to_string(),
                code: "644".to_string(),
                legacy_code: String::new(),
                pigments: IndexSet::from(["PW6".to_string()]),
                hex_color: "#F5F5F0".to_string(),
                tint_hex: None,
                opacity: Opacity::Opaque,
                drying_class: DryingClass::Medium,
                tinting_strength: 0.7,
                color_derived: false,
                pigments_verified: true,
                user_added: false,
                in_stock: true,
                notes: String::new(),
                system: palettier_domain::paint::PaintSystem::Oil,
            },
            Paint {
                id: Some(2),
                brand: "Winsor & Newton".to_string(),
                name: "Yellow Ochre".to_string(),
                code: "744".to_string(),
                legacy_code: String::new(),
                pigments: IndexSet::from(["PY43".to_string()]),
                hex_color: "#C89632".to_string(),
                tint_hex: None,
                opacity: Opacity::SemiOpaque,
                drying_class: DryingClass::Medium,
                tinting_strength: 0.5,
                color_derived: false,
                pigments_verified: true,
                user_added: false,
                in_stock: true,
                notes: String::new(),
                system: palettier_domain::paint::PaintSystem::Oil,
            },
            Paint {
                id: Some(3),
                brand: "Winsor & Newton".to_string(),
                name: "Cadmium Scarlet".to_string(),
                code: "106".to_string(),
                legacy_code: String::new(),
                pigments: IndexSet::from(["PR108".to_string()]),
                hex_color: "#E33522".to_string(),
                tint_hex: None,
                opacity: Opacity::Opaque,
                drying_class: DryingClass::VerySlow,
                tinting_strength: 0.85,
                color_derived: false,
                pigments_verified: true,
                user_added: false,
                in_stock: true,
                notes: String::new(),
                system: palettier_domain::paint::PaintSystem::Oil,
            },
            Paint {
                id: Some(4),
                brand: "Winsor & Newton".to_string(),
                name: "Ivory Black".to_string(),
                code: "331".to_string(),
                legacy_code: String::new(),
                pigments: IndexSet::from(["PBk9".to_string()]),
                hex_color: "#221F1C".to_string(),
                tint_hex: Some("#878E96".to_string()),
                opacity: Opacity::SemiOpaque,
                drying_class: DryingClass::VerySlow,
                tinting_strength: 0.6,
                color_derived: false,
                pigments_verified: true,
                user_added: false,
                in_stock: true,
                notes: String::new(),
                system: palettier_domain::paint::PaintSystem::Oil,
            },
        ]
    }

    #[test]
    fn strict_and_openapi_schemas_transform_recursively() {
        let strict = strict_schema(&PLAN_SCHEMA);
        assert_eq!(
            Some(&Value::Bool(false)),
            strict.get("additionalProperties")
        );
        assert_eq!(
            Some(&Value::Bool(false)),
            strict.pointer("/properties/zones/items/additionalProperties")
        );

        let openapi = open_api_schema(&strict);
        assert!(openapi.get("additionalProperties").is_none());
        assert!(
            openapi
                .pointer("/properties/zones/items/additionalProperties")
                .is_none()
        );
    }

    #[tokio::test]
    async fn ai_settings_redact_secret_and_persist_per_provider() {
        let settings = AiSettings::new(
            AiProvider::OpenAi,
            "https://api.openai.com",
            "gpt-5.4-mini",
            "sk-secret-key-12345",
        );
        let debug_str = format!("{settings:?}");
        assert!(!debug_str.contains("sk-secret-key-12345"));
        assert!(debug_str.contains("[REDACTED]"));
        assert!(settings.is_ready());
        assert_eq!(
            "https://api.openai.com",
            settings.validated_base_url().unwrap()
        );

        let bad_url = AiSettings::new(AiProvider::Ollama, "file:///etc/passwd", "gemma4:e4b", "");
        assert!(bad_url.validated_base_url().is_err());

        let remote_http_keyed = AiSettings::new(
            AiProvider::OpenAi,
            "http://api.example.com",
            "gpt-5.4-mini",
            "sk-secret",
        );
        assert!(remote_http_keyed.validated_base_url().is_err());

        let local_http_keyed = AiSettings::new(
            AiProvider::OpenAi,
            "http://127.0.0.1:8080",
            "gpt-5.4-mini",
            "sk-secret",
        );
        assert!(local_http_keyed.validated_base_url().is_ok());

        let userinfo_url = AiSettings::new(
            AiProvider::OpenAi,
            "https://user:pass@api.openai.com",
            "gpt-5.4-mini",
            "sk-secret",
        );
        assert!(userinfo_url.validated_base_url().is_err());

        let store = AiSettingsStore::new(Arc::new(MemSettings::default()));
        store.save(&settings).await.unwrap();
        let loaded = store.load().await.unwrap();
        assert_eq!(AiProvider::OpenAi, loaded.provider);
        assert_eq!("sk-secret-key-12345", loaded.api_key_str());

        let ollama = store.load_for(AiProvider::Ollama).await.unwrap();
        assert_eq!(AiProvider::Ollama, ollama.provider);
        assert_eq!("http://localhost:11434", ollama.base_url);
    }

    #[test]
    fn plan_enricher_interpolates_missing_steps_and_computes_mixes() {
        let palette = zorn_palette();
        let draft = PlanDraft {
            subject: "Buste de grognard".to_string(),
            general_approach: "Par glacis successifs".to_string(),
            zones: vec![ZoneDraft {
                name: "Visage".to_string(),
                material: "Peau".to_string(),
                shadow1: Some(LayerDraft {
                    role: "Ombre 1".to_string(),
                    target_hex: "#4A2E24".to_string(),
                    technique: "Glacis".to_string(),
                    note: String::new(),
                }),
                shadow2: None,
                base: Some(LayerDraft {
                    role: "Base".to_string(),
                    target_hex: "#C88E70".to_string(),
                    technique: "Aplat fin".to_string(),
                    note: String::new(),
                }),
                highlight1: None,
                highlight2: Some(LayerDraft {
                    role: "Lumière 2".to_string(),
                    target_hex: "#F2D8C4".to_string(),
                    technique: "Rehaut".to_string(),
                    note: String::new(),
                }),
                accents: vec![LayerDraft {
                    role: "Joues".to_string(),
                    target_hex: "#C95244".to_string(),
                    technique: "Pointillé".to_string(),
                    note: "Pommettes".to_string(),
                }],
                note: String::new(),
            }],
            warnings: Vec::new(),
        };

        let plan = PlanEnricher::enrich(&draft, "Zorn", &palette, palettier_domain::i18n::Locale::Fr);
        assert_eq!(1, plan.zones.len());
        let zone = &plan.zones[0];
        let ladder = zone.layers();
        assert_eq!(5, ladder.len());
        assert_eq!(1, zone.accents.len());

        // Each layer received a mix suggestion from the Kubelka-Munk solver
        for layer in ladder.iter().chain(zone.accents.iter()) {
            assert!(layer.recipe.is_some(), "couche sans recette: {:?}", layer);
        }

        let empty_draft = PlanDraft {
            subject: String::new(),
            general_approach: "Glazing".to_string(),
            zones: vec![ZoneDraft {
                name: "Face".to_string(),
                material: "Skin".to_string(),
                ..Default::default()
            }],
            warnings: Vec::new(),
        };
        let en_plan = PlanEnricher::enrich(
            &empty_draft,
            "Zorn",
            &palette,
            palettier_domain::i18n::Locale::En,
        );
        assert_eq!("Unspecified subject", en_plan.subject);
        let layers = en_plan.zones[0].layers();
        let en_roles: Vec<&str> = layers.iter().map(|l| l.role.as_str()).collect();
        assert_eq!(
            vec!["Shadow 1", "Shadow 2", "Base", "Highlight 1", "Highlight 2"],
            en_roles
        );
    }

    #[tokio::test]
    async fn painting_plan_service_parses_markdown_wrapped_json_and_enriches() {
        let json_reply = r##"Voici le plan :
```json
{
  "subject": "Officier",
  "generalApproach": "Humide sur humide",
  "zones": [
    {
      "name": "Tunique",
      "material": "Drap",
      "shadow1": { "role": "Ombre 1", "targetHex": "#202020", "technique": "Aplat", "note": "" },
      "shadow2": { "role": "Ombre 2", "targetHex": "#404040", "technique": "Fondu", "note": "" },
      "base": { "role": "Base", "targetHex": "#707070", "technique": "Fondu", "note": "" },
      "highlight1": { "role": "Lumière 1", "targetHex": "#A0A0A0", "technique": "Fondu", "note": "" },
      "highlight2": { "role": "Lumière 2", "targetHex": "#D0D0D0", "technique": "Rehaut", "note": "" },
      "accents": [],
      "note": ""
    }
  ],
  "warnings": []
}
```"##;
        let engine = Arc::new(StubEngine {
            response: json_reply.to_string(),
        });
        let service = PaintingPlanService::new(engine);
        let palette = zorn_palette();
        let sampled = vec![SampledZone::new(
            "Tunique",
            "Drap",
            Rgb::of_hex("#707070").unwrap(),
        )];

        let plan = service
            .plan(None, "Officier 75mm", "Zorn", &palette, &sampled)
            .await
            .unwrap();
        assert_eq!("Officier", plan.subject);
        assert_eq!(1, plan.zones.len());
        assert_eq!(5, plan.zones[0].layers().len());
    }

    #[test]
    fn tube_recognition_parses_and_matches_catalog() {
        let raw = json!({
            "tubes": [
                {
                    "brand": "Winsor & Newton",
                    "name": "Artists' Oil Colour Ivory Black 37ml",
                    "code": "331",
                    "pigments": "PBk9"
                },
                {
                    "brand": "Marque Inconnue",
                    "name": "Bleu Imaginaire",
                    "code": "999",
                    "pigments": ""
                }
            ],
            "notes": "Un tube retourné"
        })
        .to_string();

        let parsed = TubeRecognitionService::parse(&raw).unwrap();
        assert_eq!(2, parsed.tubes.len());
        assert_eq!("Un tube retourné", parsed.notes);
    }

    #[tokio::test]
    async fn tube_recognition_supports_acrylic_english_and_preserves_png_mime() {
        use palettier_domain::i18n::Locale;
        use palettier_domain::image::PixelMap;
        use palettier_domain::paint::PaintSystem;

        struct CaptureEngine {
            captured_system: Mutex<String>,
            captured_mime: Mutex<String>,
            response: String,
        }

        #[async_trait]
        impl ChatEngine for CaptureEngine {
            async fn complete(
                &self,
                system: &str,
                _user: &str,
                images: &[ImageAttachment],
                _schema: Option<&Value>,
            ) -> DomainResult<String> {
                *self.captured_system.lock().unwrap() = system.to_string();
                *self.captured_mime.lock().unwrap() = images[0].mime_type.clone();
                Ok(self.response.clone())
            }
        }

        let png = palettier_image::encode_png(&PixelMap::new(2, 2, vec![0xFF9A_1115; 4]).unwrap())
            .unwrap();
        let engine = Arc::new(CaptureEngine {
            captured_system: Mutex::new(String::new()),
            captured_mime: Mutex::new(String::new()),
            response: json!({
                "tubes": [{
                    "brand": "Citadel",
                    "name": "Mephiston Red",
                    "code": "",
                    "pigments": ""
                }],
                "notes": ""
            })
            .to_string(),
        });

        let mut acrylic = zorn_palette()[0].clone();
        acrylic.brand = "Citadel".to_string();
        acrylic.name = "Mephiston Red".to_string();
        acrylic.system = PaintSystem::Acrylic;
        let mut oil = acrylic.clone();
        oil.system = PaintSystem::Oil;

        let service = TubeRecognitionService::new(engine.clone());
        let res = service
            .recognize_in(&png, &[oil, acrylic], PaintSystem::Acrylic, Locale::En)
            .await
            .unwrap();

        assert!(engine.captured_system.lock().unwrap().contains("acrylic"));
        assert_eq!("image/png", *engine.captured_mime.lock().unwrap());
        assert_eq!(1, res.tubes.len());
        assert_eq!(
            Some(PaintSystem::Acrylic),
            res.tubes[0].matched.as_ref().map(|p| p.system)
        );
    }
}
