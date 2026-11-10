use palettier_domain::error::{DomainError, DomainResult};
use palettier_domain::port::SettingsRepository;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;
use url::Url;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AiProvider {
    Ollama,
    OpenAi,
    Gemini,
}

impl AiProvider {
    pub const ALL: [Self; 3] = [Self::Ollama, Self::OpenAi, Self::Gemini];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ollama => "OLLAMA",
            Self::OpenAi => "OPEN_AI",
            Self::Gemini => "GEMINI",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Ollama => "Ollama (local)",
            Self::OpenAi => "OpenAI",
            Self::Gemini => "Google Gemini",
        }
    }

    pub fn default_url(&self) -> &'static str {
        match self {
            Self::Ollama => "http://localhost:11434",
            Self::OpenAi => "https://api.openai.com",
            Self::Gemini => "https://generativelanguage.googleapis.com",
        }
    }

    pub fn default_model(&self) -> &'static str {
        match self {
            Self::Ollama => "gemma4:e4b",
            Self::OpenAi => "gpt-5.4-mini",
            Self::Gemini => "gemini-2.5-flash",
        }
    }

    pub fn needs_api_key(&self) -> bool {
        match self {
            Self::Ollama => false,
            Self::OpenAi | Self::Gemini => true,
        }
    }
}

impl fmt::Display for AiProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for AiProvider {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "OLLAMA" => Ok(Self::Ollama),
            "OPEN_AI" | "OPENAI" => Ok(Self::OpenAi),
            "GEMINI" => Ok(Self::Gemini),
            other => Err(DomainError::InvalidArgument(format!(
                "Fournisseur IA inconnu: {other}"
            ))),
        }
    }
}

/// AI assistant settings. The API key is protected by `SecretString` (redacted in `Debug` and zeroed in memory).
#[derive(Clone)]
pub struct AiSettings {
    pub provider: AiProvider,
    pub base_url: String,
    pub model: String,
    pub api_key: SecretString,
}

impl fmt::Debug for AiSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AiSettings")
            .field("provider", &self.provider)
            .field("base_url", &self.base_url)
            .field("model", &self.model)
            .field("api_key", &"[REDACTED]")
            .finish()
    }
}

impl PartialEq for AiSettings {
    fn eq(&self, other: &Self) -> bool {
        self.provider == other.provider
            && self.base_url == other.base_url
            && self.model == other.model
            && self.api_key.expose_secret() == other.api_key.expose_secret()
    }
}

impl Eq for AiSettings {}

impl Default for AiSettings {
    fn default() -> Self {
        Self::for_provider(AiProvider::Ollama)
    }
}

impl AiSettings {
    pub fn new(
        provider: AiProvider,
        base_url: impl Into<String>,
        model: impl Into<String>,
        api_key: impl Into<String>,
    ) -> Self {
        Self {
            provider,
            base_url: base_url.into(),
            model: model.into(),
            api_key: SecretString::from(api_key.into()),
        }
    }

    pub fn for_provider(provider: AiProvider) -> Self {
        Self::new(
            provider,
            provider.default_url(),
            provider.default_model(),
            "",
        )
    }

    pub fn api_key_str(&self) -> &str {
        self.api_key.expose_secret()
    }

    pub fn is_ready(&self) -> bool {
        !self.model.trim().is_empty()
            && (!self.provider.needs_api_key() || !self.api_key_str().trim().is_empty())
    }

    /// Validates the base URL (http/https schemes only).
    pub fn validated_base_url(&self) -> DomainResult<String> {
        let trimmed = self.base_url.trim().trim_end_matches('/');
        let parsed = Url::parse(trimmed).map_err(|e| {
            DomainError::InvalidArgument(format!("URL du service IA invalide ({trimmed}): {e}"))
        })?;
        match parsed.scheme() {
            "http" | "https" => Ok(trimmed.to_string()),
            scheme => Err(DomainError::InvalidArgument(format!(
                "Schéma d'URL non autorisé pour le service IA: {scheme}"
            ))),
        }
    }
}

pub struct AiSettingsStore {
    repository: Arc<dyn SettingsRepository>,
}

impl AiSettingsStore {
    pub const PROVIDER_KEY: &'static str = "ai.provider";

    pub fn new(repository: Arc<dyn SettingsRepository>) -> Self {
        Self { repository }
    }

    pub async fn load(&self) -> DomainResult<AiSettings> {
        let provider = self
            .repository
            .get(Self::PROVIDER_KEY)
            .await?
            .and_then(|s| AiProvider::from_str(&s).ok())
            .unwrap_or(AiProvider::Ollama);
        self.load_for(provider).await
    }

    pub async fn load_for(&self, provider: AiProvider) -> DomainResult<AiSettings> {
        let base_url = self
            .repository
            .get(&Self::url_key(provider))
            .await?
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| provider.default_url().to_string());
        let model = self
            .repository
            .get(&Self::model_key(provider))
            .await?
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| provider.default_model().to_string());
        let api_key = self
            .repository
            .get(&Self::api_key_key(provider))
            .await?
            .unwrap_or_default();

        Ok(AiSettings::new(provider, base_url, model, api_key))
    }

    pub async fn save(&self, settings: &AiSettings) -> DomainResult<()> {
        let p = settings.provider;
        self.repository
            .put(Self::PROVIDER_KEY, p.as_str())
            .await?;
        self.repository
            .put(&Self::url_key(p), settings.base_url.trim())
            .await?;
        self.repository
            .put(&Self::model_key(p), settings.model.trim())
            .await?;
        let key_str = settings.api_key_str().trim();
        if key_str.is_empty() {
            self.repository.remove(&Self::api_key_key(p)).await?;
        } else {
            self.repository.put(&Self::api_key_key(p), key_str).await?;
        }
        Ok(())
    }

    fn url_key(p: AiProvider) -> String {
        format!("ai.{}.url", p.as_str().to_lowercase())
    }

    fn model_key(p: AiProvider) -> String {
        format!("ai.{}.model", p.as_str().to_lowercase())
    }

    fn api_key_key(p: AiProvider) -> String {
        format!("ai.{}.apiKey", p.as_str().to_lowercase())
    }
}
