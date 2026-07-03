use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{env, fmt};

pub mod config;
pub use config::GlobalConfig;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LlmProvider {
    Anthropic,
    OpenAi,
    OpenAiCompatible,
    OpenRouter,
    Offline,
}

impl LlmProvider {
    pub fn parse(value: &str) -> Result<Self> {
        match value.to_ascii_lowercase().as_str() {
            "anthropic" | "claude" => Ok(Self::Anthropic),
            "openai" => Ok(Self::OpenAi),
            "openai-compatible" | "compatible" | "local" | "lmstudio" | "ollama" => {
                Ok(Self::OpenAiCompatible)
            }
            "openrouter" => Ok(Self::OpenRouter),
            "offline" | "none" => Ok(Self::Offline),
            other => Err(anyhow!("unknown LLM provider: {other}")),
        }
    }
}

impl fmt::Display for LlmProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Anthropic => write!(f, "anthropic"),
            Self::OpenAi => write!(f, "openai"),
            Self::OpenAiCompatible => write!(f, "openai-compatible"),
            Self::OpenRouter => write!(f, "openrouter"),
            Self::Offline => write!(f, "offline"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LlmConfig {
    pub provider: LlmProvider,
    pub model: String,
    pub base_url: String,
    pub api_key_env: Option<String>,
    pub api_key: Option<String>,
}

impl LlmConfig {
    pub fn from_env(provider_override: Option<&str>, model_override: Option<&str>) -> Result<Self> {
        if let Some(value) = provider_override {
            return Ok(Self::build_config(
                LlmProvider::parse(value)?,
                model_override,
            ));
        }

        // 1. Try environment variables first
        if let Ok(env_provider) = env::var("CERBERUS_LLM_PROVIDER") {
            return Ok(Self::build_config(
                LlmProvider::parse(&env_provider)?,
                model_override,
            ));
        }

        // 2. Fall back to global config
        if let Ok(global_cfg) = GlobalConfig::load() {
            return global_cfg.to_llm_config();
        }

        // 3. Fail if neither is found
        Err(anyhow::anyhow!(
            "Configuration not found. Please run setup."
        ))
    }

    fn build_config(provider: LlmProvider, model_override: Option<&str>) -> Self {
        match provider {
            LlmProvider::Anthropic => Self {
                provider,
                model: model_override
                    .map(ToOwned::to_owned)
                    .or_else(|| env::var("ANTHROPIC_MODEL").ok())
                    .unwrap_or_else(|| "claude-sonnet-4-5".to_string()),
                base_url: env::var("ANTHROPIC_BASE_URL")
                    .unwrap_or_else(|_| "https://api.anthropic.com".to_string()),
                api_key_env: Some("ANTHROPIC_API_KEY".to_string()),
                api_key: None,
            },
            LlmProvider::OpenAi => Self {
                provider,
                model: model_override
                    .map(ToOwned::to_owned)
                    .or_else(|| env::var("OPENAI_MODEL").ok())
                    .unwrap_or_else(|| "gpt-5".to_string()),
                base_url: env::var("OPENAI_BASE_URL")
                    .unwrap_or_else(|_| "https://api.openai.com".to_string()),
                api_key_env: Some("OPENAI_API_KEY".to_string()),
                api_key: None,
            },
            LlmProvider::OpenRouter => Self {
                provider,
                model: model_override
                    .map(ToOwned::to_owned)
                    .or_else(|| env::var("OPENROUTER_MODEL").ok())
                    .unwrap_or_else(|| "openai/gpt-4o-mini".to_string()),
                base_url: env::var("OPENROUTER_BASE_URL")
                    .unwrap_or_else(|_| "https://openrouter.ai/api".to_string()),
                api_key_env: Some("OPENROUTER_API_KEY".to_string()),
                api_key: None,
            },
            LlmProvider::OpenAiCompatible => Self {
                provider,
                model: model_override
                    .map(ToOwned::to_owned)
                    .or_else(|| env::var("CERBERUS_LLM_MODEL").ok())
                    .unwrap_or_else(|| "qwen3.5:0.8b".to_string()),
                base_url: env::var("CERBERUS_LLM_BASE_URL")
                    .unwrap_or_else(|_| "http://127.0.0.1:11434".to_string()),
                api_key_env: None,
                api_key: None,
            },
            LlmProvider::Offline => Self {
                provider,
                model: "offline".to_string(),
                base_url: "offline".to_string(),
                api_key_env: None,
                api_key: None,
            },
        }
    }

    pub fn api_key_present(&self) -> bool {
        if self.api_key.is_some() {
            return true;
        }
        self.api_key_env
            .as_ref()
            .and_then(|name| env::var(name).ok())
            .is_some_and(|value| !value.trim().is_empty())
    }

    pub fn status_lines(&self) -> Vec<String> {
        vec![
            format!("provider: {}", self.provider),
            format!("model: {}", self.model),
            format!("base_url: {}", self.base_url),
            format!(
                "api_key: {}",
                match &self.api_key_env {
                    Some(name) if self.api_key_present() => format!("{name} present"),
                    Some(name) => format!("{name} missing"),
                    None => "not required".to_string(),
                }
            ),
        ]
    }
}

pub struct LlmClient {
    config: LlmConfig,
    http: reqwest::Client,
}

impl LlmClient {
    pub fn new(config: LlmConfig) -> Self {
        Self {
            config,
            http: reqwest::Client::new(),
        }
    }

    pub async fn ask(&self, prompt: &str) -> Result<String> {
        self.ask_with_system(&cerberus_persona(), prompt).await
    }

    pub async fn ask_with_system(&self, system: &str, prompt: &str) -> Result<String> {
        match self.config.provider {
            LlmProvider::Offline => {
                Ok("Cerberus is offline. Configure CERBERUS_LLM_PROVIDER.".to_string())
            }
            LlmProvider::Anthropic => self.ask_anthropic(system, prompt).await,
            LlmProvider::OpenAi | LlmProvider::OpenAiCompatible | LlmProvider::OpenRouter => {
                self.ask_openai(system, prompt).await
            }
        }
    }

    async fn ask_anthropic(&self, system: &str, prompt: &str) -> Result<String> {
        let key = self.required_key()?;
        let response: Value = self
            .http
            .post(format!(
                "{}/v1/messages",
                self.config.base_url.trim_end_matches('/')
            ))
            .header("x-api-key", key)
            .header("anthropic-version", "2023-06-01")
            .json(&json!({
                "model": self.config.model,
                "max_tokens": 1024,
                "system": system,
                "messages": [{ "role": "user", "content": prompt }]
            }))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(response["content"][0]["text"]
            .as_str()
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| response.to_string()))
    }

    async fn ask_openai(&self, system: &str, prompt: &str) -> Result<String> {
        let mut request = self
            .http
            .post(format!(
                "{}/v1/chat/completions",
                self.config.base_url.trim_end_matches('/')
            ))
            .json(&json!({
                "model": self.config.model,
                "stream": false,
                "messages": [
                    { "role": "system", "content": system },
                    { "role": "user", "content": prompt }
                ]
            }));

        if let Some(key) = self.optional_key() {
            request = request.bearer_auth(key);
        }

        let response: Value = request.send().await?.error_for_status()?.json().await?;
        
        let message = &response["choices"][0]["message"];
        let content = message["content"].as_str().unwrap_or("").trim();
        let reasoning = message["reasoning"].as_str().unwrap_or("").trim();

        if content.is_empty() && !reasoning.is_empty() {
            return Ok(format!("*(The model ran out of tokens while thinking. Here is its internal reasoning)*\n\n{}", reasoning));
        }

        Ok(content.to_string())
    }

    fn required_key(&self) -> Result<String> {
        self.optional_key()
            .ok_or_else(|| anyhow!("missing API key env var: {:?}", self.config.api_key_env))
    }

    fn optional_key(&self) -> Option<String> {
        if let Some(key) = &self.config.api_key {
            if !key.trim().is_empty() {
                return Some(key.clone());
            }
        }
        self.config
            .api_key_env
            .as_ref()
            .and_then(|name| env::var(name).ok())
            .filter(|value| !value.trim().is_empty())
    }
}

pub fn cerberus_persona() -> String {
    "You are Cerberus, an automated security analysis engine.
Your role is to act as a strict security reviewer for code diffs and to generate automated security tests.
You strictly enforce OWASP Top 10, SOC2 compliance, and AI/ML Security guidelines.
You must specifically scan for and highlight:
1. Scripting Attacks (XSS, SQLi, Command Injection).
2. Modern OWASP 2025 Threats (Software Supply Chain Failures, Mishandling of Exceptional Conditions).
3. LLM-specific Threats (Prompt Injection, System Prompt Leakage, Excessive Agency, Unbounded Consumption).
4. Adversarial Data Poisoning (in AI pipelines).
When given a code diff, you MUST output a raw JSON array of vulnerabilities. Do not use conversational filler or markdown blocks.
JSON Format:
[
  { 
    \"severity\": \"Low|Medium|High|Critical\", 
    \"description\": \"<string>\", 
    \"remediation\": \"<string>\", 
    \"file\": \"<string>\",
    \"original_code\": \"<exact matching vulnerable lines from the file to replace>\",
    \"replacement_code\": \"<secure code to insert>\"
  }
]
If there are no vulnerabilities, output an empty array: []
Be extremely concise, output only actionable JSON, and never break character."
        .to_string()
}
