//! Tier-aware LLM client for the Witness Dyad.
//!
//! Provider selection (highest precedence first):
//!   1. `LLM_PROVIDER=nvidia|openrouter|omniroute` with that provider configured
//!   2. OmniRoute configured (`OMNIROUTE_API_KEY` + `OMNIROUTE_MODEL`) → OmniRoute,
//!      the default gateway at `OMNIROUTE_BASE_URL` (default
//!      `https://omniroute.thoughtseed.space/v1`)
//!   3. `NVIDIA_API_KEY` set, no `OPENROUTER_API_KEY` → NVIDIA (auto)
//!   4. `OPENROUTER_API_KEY` set → OpenRouter (also the compat default when both
//!      NVIDIA and OpenRouter keys are set)
//!   5. Nothing configured → silent fallback to rule-based dyad
//!
//! Model selection is tier-aware (free/standard/enterprise → NVIDIA routing
//! table); OmniRoute uses `OMNIROUTE_MODEL` for every role because its model
//! ids are deployment-specific. A failed OmniRoute call retries via NVIDIA, then
//! OpenRouter; a failed NVIDIA call retries via OpenRouter.
//!
//! Long-form writing (`for_reading` + `complete_with_budget`) streams the
//! response: OmniRoute closes non-streamed requests after 30 seconds, and a
//! report section can take minutes. `OMNIROUTE_READING_MODEL` picks a writing
//! model for readings, falling back to `OMNIROUTE_MODEL`.

use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;
use tracing::{debug, warn};

const NVIDIA_BASE_URL: &str = "https://integrate.api.nvidia.com/v1";
const OPENROUTER_BASE_URL: &str = "https://openrouter.ai/api/v1";
const OMNIROUTE_DEFAULT_BASE_URL: &str = "https://omniroute.thoughtseed.space/v1";
/// Upper bound for one non-streamed completion.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(300);
/// Upper bound for one streamed completion (a long report section).
const STREAM_TIMEOUT: Duration = Duration::from_secs(20 * 60);
/// A stream that sends nothing (not even a keep-alive) for this long is dead.
const STREAM_IDLE_TIMEOUT: Duration = Duration::from_secs(120);

// Tier → (model_id, max_tokens) for the Witness Dyad.
// Mirrors nvidia-routing.ts — single source of truth is the TS file;
// update both when swapping models.
fn model_for_tier(tier: &str, role: &str) -> (&'static str, u32) {
    match (tier, role) {
        // enterprise: deep reasoning models
        ("enterprise", "aletheios") => ("nvidia/llama-3.3-nemotron-super-49b-v1.5", 2048),
        ("enterprise", "pichet") => ("minimaxai/minimax-m2.7", 2048),
        ("enterprise", "synthesis") => ("openai/gpt-oss-120b", 2048),
        // standard/subscriber: autoresearch winners (gpt-oss-120b both roles)
        ("standard", "aletheios") => ("openai/gpt-oss-120b", 1536),
        ("standard", "pichet") => ("openai/gpt-oss-120b", 1536),
        ("standard", "synthesis") => ("z-ai/glm4.7", 1536),
        // free: fast, lightweight
        ("free", "aletheios") => ("moonshotai/kimi-k2-instruct", 512),
        ("free", "pichet") => ("minimaxai/minimax-m2.7", 1024),
        ("free", "synthesis") => ("moonshotai/kimi-k2-instruct", 512),
        // default (unknown tier → standard)
        (_, "pichet") => ("openai/gpt-oss-120b", 1536),
        (_, "synthesis") => ("openai/gpt-oss-120b", 1024),
        _ => ("openai/gpt-oss-120b", 1536),
    }
}

/// OmniRoute gateway settings; present only when both key and model are set.
struct OmniRouteConfig {
    base_url: String,
    api_key: String,
    model: String,
}

impl OmniRouteConfig {
    fn from_env() -> Option<Self> {
        let non_empty = |name: &str| env::var(name).ok().filter(|v| !v.trim().is_empty());
        Some(Self {
            base_url: non_empty("OMNIROUTE_BASE_URL")
                .unwrap_or_else(|| OMNIROUTE_DEFAULT_BASE_URL.to_string())
                .trim_end_matches('/')
                .to_string(),
            api_key: non_empty("OMNIROUTE_API_KEY")?,
            model: non_empty("OMNIROUTE_MODEL")?,
        })
    }
}

/// A completed chat call and where it was answered.
#[derive(Clone, Debug)]
pub struct Completion {
    pub text: String,
    pub provider: LlmProvider,
    /// The model id sent to the provider (OmniRoute's are deployment-specific).
    pub model: String,
    /// The model the provider reports having used, when it says.
    pub served_model: Option<String>,
}

pub struct LlmClient {
    client: reqwest::Client,
    omniroute: Option<OmniRouteConfig>,
    nvidia_api_key: Option<String>,
    openrouter_api_key: Option<String>,
    /// Active provider for this client instance.
    pub provider: LlmProvider,
    /// User tier (free / standard / enterprise) — drives model selection.
    pub tier: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LlmProvider {
    OmniRoute,
    Nvidia,
    OpenRouter,
}

/// Pick the primary provider from what is configured. Pure for testability.
fn select_provider(
    provider_env: &str,
    has_omniroute: bool,
    has_nvidia: bool,
    has_openrouter: bool,
) -> Option<LlmProvider> {
    match provider_env {
        "omniroute" if has_omniroute => Some(LlmProvider::OmniRoute),
        "nvidia" if has_nvidia => Some(LlmProvider::Nvidia),
        "openrouter" if has_openrouter => Some(LlmProvider::OpenRouter),
        _ if has_omniroute => Some(LlmProvider::OmniRoute),
        _ if has_nvidia && !has_openrouter => Some(LlmProvider::Nvidia),
        // Covers both "OpenRouter only" and "NVIDIA + OpenRouter" — OpenRouter
        // is the default for the latter (compat with witness-agents).
        _ if has_openrouter => Some(LlmProvider::OpenRouter),
        _ => None, // nothing configured → silent fallback
    }
}

/// The provider to retry with after `primary` fails, if one is configured.
fn fallback_provider(
    primary: &LlmProvider,
    has_nvidia: bool,
    has_openrouter: bool,
) -> Option<LlmProvider> {
    match primary {
        LlmProvider::OmniRoute if has_nvidia => Some(LlmProvider::Nvidia),
        LlmProvider::OmniRoute | LlmProvider::Nvidia if has_openrouter => {
            Some(LlmProvider::OpenRouter)
        }
        _ => None,
    }
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: u32,
    temperature: f32,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    stream: bool,
}

#[derive(Serialize, Deserialize, Clone)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
    #[serde(default)]
    model: Option<String>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatResponseMessage,
}

#[derive(Deserialize)]
struct ChatResponseMessage {
    content: Option<String>,
    // Reasoning models (NVIDIA NIM) sometimes put their answer here when
    // chain-of-thought exhausts the content token budget.
    reasoning_content: Option<String>,
}

impl LlmClient {
    /// Build a tier-aware client from environment.
    /// Returns `None` when no API key is available → caller falls back to rule-based dyad.
    pub fn for_tier(tier: &str) -> Option<Self> {
        let omniroute = OmniRouteConfig::from_env();
        let nvidia_key = env::var("NVIDIA_API_KEY").ok();
        let openrouter_key = env::var("OPENROUTER_API_KEY").ok();
        let provider_env = env::var("LLM_PROVIDER").unwrap_or_default().to_lowercase();

        let provider = select_provider(
            &provider_env,
            omniroute.is_some(),
            nvidia_key.is_some(),
            openrouter_key.is_some(),
        )?;

        Some(Self {
            client: reqwest::Client::new(),
            omniroute,
            nvidia_api_key: nvidia_key,
            openrouter_api_key: openrouter_key,
            provider,
            tier: tier.to_string(),
        })
    }

    /// Like `for_tier`, with `OMNIROUTE_READING_MODEL` (when set) as the
    /// OmniRoute model, for long-form readings.
    pub fn for_reading(tier: &str) -> Option<Self> {
        let mut client = Self::for_tier(tier)?;
        let reading_model = env::var("OMNIROUTE_READING_MODEL")
            .ok()
            .filter(|v| !v.trim().is_empty());
        if let (Some(cfg), Some(model)) = (client.omniroute.as_mut(), reading_model) {
            cfg.model = model;
        }
        Some(client)
    }

    /// A client for an explicit OmniRoute gateway, without reading the
    /// environment or configuring fallbacks.
    pub fn omniroute(base_url: &str, api_key: &str, model: &str, tier: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            omniroute: Some(OmniRouteConfig {
                base_url: base_url.trim_end_matches('/').to_string(),
                api_key: api_key.to_string(),
                model: model.to_string(),
            }),
            nvidia_api_key: None,
            openrouter_api_key: None,
            provider: LlmProvider::OmniRoute,
            tier: tier.to_string(),
        }
    }

    /// Send a chat completion for a specific agent role.
    /// Tries the primary provider first, then each configured fallback.
    pub async fn complete_for_role(
        &self,
        role: &str,
        system: &str,
        user: &str,
    ) -> Result<String, String> {
        let (_, max_tokens) = model_for_tier(&self.tier, role);
        self.complete(role, system, user, max_tokens, false)
            .await
            .map(|completion| completion.text)
    }

    /// Like `complete_for_role`, with the caller's output budget, streamed
    /// so it can run for minutes, reporting which provider and model answered.
    pub async fn complete_with_budget(
        &self,
        role: &str,
        system: &str,
        user: &str,
        max_tokens: u32,
    ) -> Result<Completion, String> {
        self.complete(role, system, user, max_tokens, true).await
    }

    async fn complete(
        &self,
        role: &str,
        system: &str,
        user: &str,
        max_tokens: u32,
        stream: bool,
    ) -> Result<Completion, String> {
        let (model, _) = model_for_tier(&self.tier, role);

        let mut provider = self.provider.clone();
        loop {
            match self
                .complete_with_provider(&provider, model, system, user, max_tokens, stream)
                .await
            {
                Ok(t) => return Ok(t),
                Err(e) => match fallback_provider(
                    &provider,
                    self.nvidia_api_key.is_some(),
                    self.openrouter_api_key.is_some(),
                ) {
                    Some(next) => {
                        warn!("[witness-llm] {provider:?} failed ({e}), retrying via {next:?}");
                        provider = next;
                    }
                    None => return Err(e),
                },
            }
        }
    }

    /// Lower-level: complete with an explicit provider.
    async fn complete_with_provider(
        &self,
        provider: &LlmProvider,
        model: &str,
        system: &str,
        user: &str,
        max_tokens: u32,
        stream: bool,
    ) -> Result<Completion, String> {
        let (base_url, api_key, model) = match provider {
            LlmProvider::OmniRoute => {
                let cfg = self
                    .omniroute
                    .as_ref()
                    .ok_or_else(|| "OmniRoute is not configured".to_string())?;
                (
                    cfg.base_url.as_str(),
                    cfg.api_key.as_str(),
                    cfg.model.as_str(),
                )
            }
            LlmProvider::Nvidia => (
                NVIDIA_BASE_URL,
                self.nvidia_api_key.as_deref().unwrap_or(""),
                model,
            ),
            LlmProvider::OpenRouter => (
                OPENROUTER_BASE_URL,
                self.openrouter_api_key.as_deref().unwrap_or(""),
                model,
            ),
        };

        let url = format!("{base_url}/chat/completions");
        let body = ChatRequest {
            model: model.to_string(),
            messages: vec![
                ChatMessage {
                    role: "system".into(),
                    content: system.into(),
                },
                ChatMessage {
                    role: "user".into(),
                    content: user.into(),
                },
            ],
            max_tokens,
            temperature: 0.82,
            stream,
        };

        debug!("[witness-llm] POST {url} model={model} provider={provider:?}");

        let mut resp = self
            .client
            .post(&url)
            .bearer_auth(api_key)
            .timeout(if stream {
                STREAM_TIMEOUT
            } else {
                REQUEST_TIMEOUT
            })
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("witness LLM request failed: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            warn!("[witness-llm] {provider:?} error {status}: {text}");
            return Err(format!("LLM API error {status}: {text}"));
        }

        if stream {
            let mut raw = Vec::new();
            loop {
                match tokio::time::timeout(STREAM_IDLE_TIMEOUT, resp.chunk()).await {
                    Err(_) => return Err("LLM stream stalled".to_string()),
                    Ok(Err(e)) => return Err(format!("LLM stream failed: {e}")),
                    Ok(Ok(None)) => break,
                    Ok(Ok(Some(bytes))) => raw.extend_from_slice(&bytes),
                }
            }
            let streamed = parse_stream(&String::from_utf8_lossy(&raw))?;
            return Ok(Completion {
                text: streamed.text,
                provider: provider.clone(),
                model: model.to_string(),
                served_model: streamed.served_model,
            });
        }

        let parsed: ChatResponse = resp
            .json()
            .await
            .map_err(|e| format!("failed to parse LLM response: {e}"))?;

        let served_model = parsed.model.filter(|m| !m.is_empty());
        parsed
            .choices
            .into_iter()
            .next()
            .and_then(|c| {
                // Reasoning models may put content in reasoning_content when budget is tight
                c.message
                    .content
                    .filter(|s| !s.is_empty())
                    .or(c.message.reasoning_content)
            })
            .map(|s| Completion {
                text: s.trim().to_string(),
                provider: provider.clone(),
                model: model.to_string(),
                served_model: served_model.clone(),
            })
            .ok_or_else(|| "LLM returned empty choices".to_string())
    }
}

/// The text and model of a streamed (server-sent events) completion.
#[derive(Debug, PartialEq)]
struct Streamed {
    text: String,
    served_model: Option<String>,
}

/// Collect a chat completion stream. Keep-alive chunks are skipped; reasoning
/// text is used only when the model sent no content.
fn parse_stream(raw: &str) -> Result<Streamed, String> {
    let mut content = String::new();
    let mut reasoning = String::new();
    let mut served_model = None;
    for line in raw.lines() {
        let Some(data) = line.strip_prefix("data:").map(str::trim) else {
            continue;
        };
        if data.is_empty() || data == "[DONE]" {
            continue;
        }
        let chunk: serde_json::Value =
            serde_json::from_str(data).map_err(|e| format!("bad LLM stream chunk: {e}"))?;
        if let Some(error) = chunk.get("error") {
            return Err(format!("LLM stream error: {error}"));
        }
        if let Some(model) = chunk["model"].as_str() {
            if !model.is_empty() && model != "keepalive" {
                served_model = Some(model.to_string());
            }
        }
        let delta = &chunk["choices"][0]["delta"];
        if let Some(text) = delta["content"].as_str() {
            content.push_str(text);
        }
        if let Some(text) = delta["reasoning_content"].as_str() {
            reasoning.push_str(text);
        }
    }
    let text = if content.trim().is_empty() {
        reasoning
    } else {
        content
    };
    if text.trim().is_empty() {
        return Err("LLM stream returned no text".to_string());
    }
    Ok(Streamed {
        text: text.trim().to_string(),
        served_model,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stream_collects_content_and_skips_keepalives() {
        let raw = concat!(
            "data: {\"id\":\"chatcmpl-keepalive\",\"model\":\"keepalive\",\"choices\":[{\"index\":0,\"delta\":{}}]}\n\n",
            "data: {\"model\":\"claude-haiku-5-5\",\"choices\":[{\"delta\":{\"content\":\"# Opening\\n\"}}]}\n\n",
            "data: {\"model\":\"claude-haiku-5-5\",\"choices\":[{\"delta\":{\"content\":\"Words.\"}}]}\n\n",
            "data: {\"model\":\"claude-haiku-5-5\",\"choices\":[{\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n",
            "data: [DONE]\n"
        );
        assert_eq!(
            parse_stream(raw).unwrap(),
            Streamed {
                text: "# Opening\nWords.".into(),
                served_model: Some("claude-haiku-5-5".into())
            }
        );
    }

    #[test]
    fn a_stream_falls_back_to_reasoning_and_reports_errors() {
        let reasoning =
            "data: {\"choices\":[{\"delta\":{\"reasoning_content\":\"Only this.\"}}]}\n";
        assert_eq!(parse_stream(reasoning).unwrap().text, "Only this.");
        assert!(
            parse_stream("data: {\"error\":{\"message\":\"upstream\"}}\n")
                .unwrap_err()
                .contains("upstream")
        );
        assert!(parse_stream("data: [DONE]\n").is_err());
    }

    #[test]
    fn omniroute_is_the_default_when_configured() {
        assert_eq!(
            select_provider("", true, true, true),
            Some(LlmProvider::OmniRoute)
        );
        assert_eq!(
            select_provider("", true, false, false),
            Some(LlmProvider::OmniRoute)
        );
    }

    #[test]
    fn explicit_provider_wins_when_configured() {
        assert_eq!(
            select_provider("nvidia", true, true, true),
            Some(LlmProvider::Nvidia)
        );
        assert_eq!(
            select_provider("openrouter", true, true, true),
            Some(LlmProvider::OpenRouter)
        );
        // Asking for an unconfigured provider falls back to auto selection.
        assert_eq!(
            select_provider("omniroute", false, true, false),
            Some(LlmProvider::Nvidia)
        );
    }

    #[test]
    fn legacy_auto_selection_is_unchanged_without_omniroute() {
        assert_eq!(
            select_provider("", false, true, false),
            Some(LlmProvider::Nvidia)
        );
        assert_eq!(
            select_provider("", false, false, true),
            Some(LlmProvider::OpenRouter)
        );
        assert_eq!(
            select_provider("", false, true, true),
            Some(LlmProvider::OpenRouter)
        );
        assert_eq!(select_provider("", false, false, false), None);
    }

    #[test]
    fn fallback_chain_is_omniroute_then_nvidia_then_openrouter() {
        let omni = LlmProvider::OmniRoute;
        assert_eq!(
            fallback_provider(&omni, true, true),
            Some(LlmProvider::Nvidia)
        );
        assert_eq!(
            fallback_provider(&omni, false, true),
            Some(LlmProvider::OpenRouter)
        );
        assert_eq!(fallback_provider(&omni, false, false), None);
        let nvidia = LlmProvider::Nvidia;
        assert_eq!(
            fallback_provider(&nvidia, true, true),
            Some(LlmProvider::OpenRouter)
        );
        assert_eq!(
            fallback_provider(&LlmProvider::OpenRouter, true, true),
            None
        );
    }
}
