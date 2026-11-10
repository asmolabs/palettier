use crate::schema::{open_api_schema, strict_schema};
use crate::settings::{AiProvider, AiSettings};
use async_trait::async_trait;
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use palettier_domain::error::{DomainError, DomainResult};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::RwLock;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ImageAttachment {
    pub bytes: Vec<u8>,
    pub mime_type: String,
}

impl ImageAttachment {
    pub fn jpeg(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            mime_type: "image/jpeg".to_string(),
        }
    }

    pub fn base64_encoded(&self) -> String {
        BASE64.encode(&self.bytes)
    }

    pub fn data_url(&self) -> String {
        format!("data:{};base64,{}", self.mime_type, self.base64_encoded())
    }
}

#[async_trait]
pub trait ChatEngine: Send + Sync {
    async fn complete(
        &self,
        system: &str,
        user: &str,
        images: &[ImageAttachment],
        schema: Option<&Value>,
    ) -> DomainResult<String>;
}

fn default_http_client() -> DomainResult<Client> {
    Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(180))
        .build()
        .map_err(|e| DomainError::Storage(format!("Impossible d'initialiser le client HTTP: {e}")))
}

/// Local Ollama engine (`/api/chat`).
pub struct OllamaEngine {
    client: Client,
    settings: AiSettings,
}

impl OllamaEngine {
    pub fn new(client: Client, settings: AiSettings) -> Self {
        Self { client, settings }
    }
}

#[async_trait]
impl ChatEngine for OllamaEngine {
    async fn complete(
        &self,
        system: &str,
        user: &str,
        images: &[ImageAttachment],
        schema: Option<&Value>,
    ) -> DomainResult<String> {
        let base_url = self.settings.validated_base_url()?;
        let url = format!("{base_url}/api/chat");

        let mut messages = Vec::new();
        if !system.trim().is_empty() {
            messages.push(json!({
                "role": "system",
                "content": system,
            }));
        }

        let encoded_images: Vec<String> = images.iter().map(ImageAttachment::base64_encoded).collect();
        let mut user_msg = json!({
            "role": "user",
            "content": user,
        });
        if !encoded_images.is_empty() {
            user_msg["images"] = json!(encoded_images);
        }
        messages.push(user_msg);

        let mut payload = json!({
            "model": self.settings.model,
            "messages": messages,
            "stream": false,
            "think": false,
            "options": {
                "temperature": 0.2
            }
        });
        if let Some(s) = schema {
            payload["format"] = s.clone();
        }

        let resp = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| DomainError::Storage(format!("Ollama injoignable ({url}): {e}")))?;

        let status = resp.status();
        let body: Value = resp
            .json()
            .await
            .map_err(|e| DomainError::Storage(format!("Réponse Ollama illisible: {e}")))?;

        if !status.is_success() {
            let msg = body
                .get("error")
                .and_then(Value::as_str)
                .unwrap_or("Erreur Ollama");
            return Err(DomainError::Storage(format!("Ollama ({status}): {msg}")));
        }

        let content = body
            .pointer("/message/content")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim();
        if content.is_empty() {
            return Err(DomainError::Storage(
                "Ollama a répondu, mais sans contenu.".to_string(),
            ));
        }
        Ok(content.to_string())
    }
}

/// OpenAI engine (`/v1/chat/completions`) with `Authorization: Bearer` and strict `json_schema`.
pub struct OpenAiEngine {
    client: Client,
    settings: AiSettings,
}

impl OpenAiEngine {
    pub fn new(client: Client, settings: AiSettings) -> Self {
        Self { client, settings }
    }
}

#[async_trait]
impl ChatEngine for OpenAiEngine {
    async fn complete(
        &self,
        system: &str,
        user: &str,
        images: &[ImageAttachment],
        schema: Option<&Value>,
    ) -> DomainResult<String> {
        let api_key = self.settings.api_key_str().trim();
        if api_key.is_empty() {
            return Err(DomainError::InvalidArgument(
                "Clé OpenAI manquante : renseignez-la dans Paramètres.".to_string(),
            ));
        }
        let base_url = self.settings.validated_base_url()?;
        let url = format!("{base_url}/v1/chat/completions");

        let mut messages = Vec::new();
        if !system.trim().is_empty() {
            messages.push(json!({
                "role": "system",
                "content": system,
            }));
        }

        let user_content = if images.is_empty() {
            json!(user)
        } else {
            let mut parts = vec![json!({ "type": "text", "text": user })];
            for img in images {
                parts.push(json!({
                    "type": "image_url",
                    "image_url": { "url": img.data_url() }
                }));
            }
            Value::Array(parts)
        };
        messages.push(json!({
            "role": "user",
            "content": user_content,
        }));

        let mut payload = json!({
            "model": self.settings.model,
            "messages": messages,
            "temperature": 0.2,
        });
        if let Some(s) = schema {
            payload["response_format"] = json!({
                "type": "json_schema",
                "json_schema": {
                    "name": "reponse",
                    "strict": true,
                    "schema": strict_schema(s),
                }
            });
        }

        let resp = self
            .client
            .post(&url)
            .bearer_auth(api_key)
            .json(&payload)
            .send()
            .await
            .map_err(|e| DomainError::Storage(format!("OpenAI injoignable ({url}): {e}")))?;

        let status = resp.status();
        let body: Value = resp
            .json()
            .await
            .map_err(|e| DomainError::Storage(format!("Réponse OpenAI illisible: {e}")))?;

        if !status.is_success() {
            let msg = body
                .pointer("/error/message")
                .and_then(Value::as_str)
                .unwrap_or("Erreur OpenAI");
            return Err(DomainError::Storage(format!("OpenAI ({status}): {msg}")));
        }

        let content = body
            .pointer("/choices/0/message/content")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim();
        if content.is_empty() {
            return Err(DomainError::Storage(
                "OpenAI a répondu, mais sans contenu.".to_string(),
            ));
        }
        Ok(content.to_string())
    }
}

/// Google Gemini engine (`/v1beta/models/{model}:generateContent`) with `x-goog-api-key` header.
pub struct GeminiEngine {
    client: Client,
    settings: AiSettings,
}

impl GeminiEngine {
    pub fn new(client: Client, settings: AiSettings) -> Self {
        Self { client, settings }
    }
}

#[async_trait]
impl ChatEngine for GeminiEngine {
    async fn complete(
        &self,
        system: &str,
        user: &str,
        images: &[ImageAttachment],
        schema: Option<&Value>,
    ) -> DomainResult<String> {
        let api_key = self.settings.api_key_str().trim();
        if api_key.is_empty() {
            return Err(DomainError::InvalidArgument(
                "Clé Gemini manquante : renseignez-la dans Paramètres.".to_string(),
            ));
        }
        let base_url = self.settings.validated_base_url()?;
        let url = format!("{base_url}/v1beta/models/{}:generateContent", self.settings.model);

        let mut parts = vec![json!({ "text": user })];
        for img in images {
            parts.push(json!({
                "inline_data": {
                    "mime_type": img.mime_type,
                    "data": img.base64_encoded(),
                }
            }));
        }

        let mut gen_config = json!({
            "temperature": 0.2,
        });
        if let Some(s) = schema {
            gen_config["responseMimeType"] = json!("application/json");
            gen_config["responseSchema"] = open_api_schema(s);
        }

        let mut payload = json!({
            "contents": [{
                "role": "user",
                "parts": parts,
            }],
            "generationConfig": gen_config,
        });
        if !system.trim().is_empty() {
            payload["system_instruction"] = json!({
                "parts": [{ "text": system }]
            });
        }

        let resp = self
            .client
            .post(&url)
            .header("x-goog-api-key", api_key)
            .json(&payload)
            .send()
            .await
            .map_err(|e| DomainError::Storage(format!("Gemini injoignable: {e}")))?;

        let status = resp.status();
        let body: Value = resp
            .json()
            .await
            .map_err(|e| DomainError::Storage(format!("Réponse Gemini illisible: {e}")))?;

        if !status.is_success() {
            let msg = body
                .pointer("/error/message")
                .and_then(Value::as_str)
                .unwrap_or("Erreur Gemini");
            return Err(DomainError::Storage(format!("Gemini ({status}): {msg}")));
        }

        let content = body
            .pointer("/candidates/0/content/parts/0/text")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim();
        if content.is_empty() {
            return Err(DomainError::Storage(
                "Gemini a répondu, mais sans contenu.".to_string(),
            ));
        }
        Ok(content.to_string())
    }
}

/// AI engine router delegating to the provider configured in `AiSettings`.
pub struct ChatEngines {
    client: Client,
    current: RwLock<AiSettings>,
}

impl ChatEngines {
    pub fn new(initial: AiSettings) -> DomainResult<Self> {
        Ok(Self {
            client: default_http_client()?,
            current: RwLock::new(initial),
        })
    }

    pub fn settings(&self) -> AiSettings {
        self.current
            .read()
            .map(|g| g.clone())
            .unwrap_or_default()
    }

    pub fn configure(&self, next: AiSettings) {
        if let Ok(mut guard) = self.current.write() {
            *guard = next;
        }
    }
}

#[async_trait]
impl ChatEngine for ChatEngines {
    async fn complete(
        &self,
        system: &str,
        user: &str,
        images: &[ImageAttachment],
        schema: Option<&Value>,
    ) -> DomainResult<String> {
        let active = self.settings();
        match active.provider {
            AiProvider::Ollama => {
                OllamaEngine::new(self.client.clone(), active)
                    .complete(system, user, images, schema)
                    .await
            }
            AiProvider::OpenAi => {
                OpenAiEngine::new(self.client.clone(), active)
                    .complete(system, user, images, schema)
                    .await
            }
            AiProvider::Gemini => {
                GeminiEngine::new(self.client.clone(), active)
                    .complete(system, user, images, schema)
                    .await
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelOption {
    pub id: String,
    pub detail: String,
}

impl ModelOption {
    pub fn new(id: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            detail: detail.into(),
        }
    }

    pub fn label(&self) -> String {
        if self.detail.is_empty() {
            self.id.clone()
        } else {
            format!("{} — {}", self.id, self.detail)
        }
    }
}

pub struct ModelCatalog {
    client: Client,
}

impl Default for ModelCatalog {
    fn default() -> Self {
        Self {
            client: default_http_client().unwrap_or_else(|_| Client::new()),
        }
    }
}

impl ModelCatalog {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn available_for(&self, settings: &AiSettings) -> DomainResult<Vec<ModelOption>> {
        match settings.provider {
            AiProvider::Ollama => self.fetch_ollama(settings).await,
            AiProvider::OpenAi => self.fetch_open_ai(settings).await,
            AiProvider::Gemini => self.fetch_gemini(settings).await,
        }
    }

    async fn fetch_ollama(&self, settings: &AiSettings) -> DomainResult<Vec<ModelOption>> {
        let base_url = settings.validated_base_url()?;
        let url = format!("{base_url}/api/tags");
        let body: Value = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| DomainError::Storage(format!("Ollama injoignable ({url}): {e}")))?
            .json()
            .await
            .map_err(|e| DomainError::Storage(format!("Réponse Ollama illisible: {e}")))?;

        let mut models = Vec::new();
        if let Some(arr) = body.get("models").and_then(Value::as_array) {
            for m in arr {
                let Some(name) = m.get("name").and_then(Value::as_str) else {
                    continue;
                };
                let size_str = m
                    .pointer("/details/parameter_size")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .or_else(|| {
                        m.get("size")
                            .and_then(Value::as_i64)
                            .filter(|&b| b > 0)
                            .map(Self::human_size)
                    })
                    .unwrap_or_default();
                models.push(ModelOption::new(name, size_str));
            }
        }
        models.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(models)
    }

    async fn fetch_open_ai(&self, settings: &AiSettings) -> DomainResult<Vec<ModelOption>> {
        let api_key = settings.api_key_str().trim();
        if api_key.is_empty() {
            return Err(DomainError::InvalidArgument("Clé OpenAI manquante.".to_string()));
        }
        let base_url = settings.validated_base_url()?;
        let url = format!("{base_url}/v1/models");
        let resp = self
            .client
            .get(&url)
            .bearer_auth(api_key)
            .send()
            .await
            .map_err(|e| DomainError::Storage(format!("OpenAI injoignable: {e}")))?;
        let status = resp.status();
        let body: Value = resp
            .json()
            .await
            .map_err(|e| DomainError::Storage(format!("Réponse OpenAI illisible: {e}")))?;
        if !status.is_success() {
            let msg = body
                .pointer("/error/message")
                .and_then(Value::as_str)
                .unwrap_or("Erreur OpenAI");
            return Err(DomainError::Storage(format!("OpenAI ({status}): {msg}")));
        }

        let mut models = Vec::new();
        if let Some(arr) = body.get("data").and_then(Value::as_array) {
            for m in arr {
                if let Some(id) = m.get("id").and_then(Value::as_str)
                    && Self::is_open_ai_chat_model(id)
                {
                    models.push(ModelOption::new(id, ""));
                }
            }
        }
        models.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(models)
    }

    async fn fetch_gemini(&self, settings: &AiSettings) -> DomainResult<Vec<ModelOption>> {
        let api_key = settings.api_key_str().trim();
        if api_key.is_empty() {
            return Err(DomainError::InvalidArgument("Clé Gemini manquante.".to_string()));
        }
        let base_url = settings.validated_base_url()?;
        let url = format!("{base_url}/v1beta/models");
        let resp = self
            .client
            .get(&url)
            .header("x-goog-api-key", api_key)
            .send()
            .await
            .map_err(|e| DomainError::Storage(format!("Gemini injoignable: {e}")))?;
        let status = resp.status();
        let body: Value = resp
            .json()
            .await
            .map_err(|e| DomainError::Storage(format!("Réponse Gemini illisible: {e}")))?;
        if !status.is_success() {
            let msg = body
                .pointer("/error/message")
                .and_then(Value::as_str)
                .unwrap_or("Erreur Gemini");
            return Err(DomainError::Storage(format!("Gemini ({status}): {msg}")));
        }

        let mut models = Vec::new();
        if let Some(arr) = body.get("models").and_then(Value::as_array) {
            for m in arr {
                let supports_generate = m
                    .get("supportedGenerationMethods")
                    .and_then(Value::as_array)
                    .is_some_and(|methods| {
                        methods.iter().any(|v| v.as_str() == Some("generateContent"))
                    });
                if !supports_generate {
                    continue;
                }
                if let Some(name) = m.get("name").and_then(Value::as_str) {
                    let id = name.strip_prefix("models/").unwrap_or(name);
                    let display = m
                        .get("displayName")
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    models.push(ModelOption::new(id, display));
                }
            }
        }
        models.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(models)
    }

    fn is_open_ai_chat_model(id: &str) -> bool {
        let chat_prefix = id.starts_with("gpt-")
            || id.starts_with("o1")
            || id.starts_with("o3")
            || id.starts_with("o4")
            || id.starts_with("chatgpt-");
        let non_text = [
            "instruct",
            "embedding",
            "whisper",
            "tts",
            "dall-e",
            "audio",
            "realtime",
            "transcribe",
            "moderation",
            "search",
            "image",
        ]
        .iter()
        .any(|frag| id.contains(frag));
        chat_prefix && !non_text
    }

    fn human_size(bytes: i64) -> String {
        let gb = (bytes as f64) / 1_073_741_824.0;
        let rounded = (gb * 10.0).round() / 10.0;
        format!("{rounded} Go").replace('.', ",")
    }
}
