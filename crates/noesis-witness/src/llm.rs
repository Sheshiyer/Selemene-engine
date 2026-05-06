//! Tier-aware LLM client for the Witness Dyad.
//!
//! Provider selection (highest precedence first):
//!   1. `LLM_PROVIDER=nvidia` → NVIDIA NIM at `https://integrate.api.nvidia.com/v1`
//!   2. `LLM_PROVIDER=openrouter` → OpenRouter at `https://openrouter.ai/api/v1`
//!   3. `NVIDIA_API_KEY` set, no `OPENROUTER_API_KEY` → NVIDIA (auto)
//!   4. `OPENROUTER_API_KEY` set, no `NVIDIA_API_KEY` → OpenRouter (auto)
//!   5. Both keys set → OpenRouter (compat default)
//!   6. Neither key → silent fallback to rule-based dyad
//!
//! Model selection is tier-aware (free/standard/enterprise → NVIDIA routing table).
//! Falls back to OpenRouter when NVIDIA call fails.

use serde::{Deserialize, Serialize};
use std::env;
use tracing::{debug, warn};

const NVIDIA_BASE_URL: &str = "https://integrate.api.nvidia.com/v1";
const OPENROUTER_BASE_URL: &str = "https://openrouter.ai/api/v1";

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

pub struct LlmClient {
    client: reqwest::Client,
    nvidia_api_key: Option<String>,
    openrouter_api_key: Option<String>,
    /// Active provider for this client instance.
    pub provider: LlmProvider,
    /// User tier (free / standard / enterprise) — drives model selection.
    pub tier: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LlmProvider {
    Nvidia,
    OpenRouter,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Serialize, Deserialize, Clone)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
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
        let nvidia_key = env::var("NVIDIA_API_KEY").ok();
        let openrouter_key = env::var("OPENROUTER_API_KEY").ok();
        let provider_env = env::var("LLM_PROVIDER").unwrap_or_default().to_lowercase();

        let provider = if provider_env == "nvidia" && nvidia_key.is_some() {
            LlmProvider::Nvidia
        } else if provider_env == "openrouter" && openrouter_key.is_some() {
            LlmProvider::OpenRouter
        } else if nvidia_key.is_some() && openrouter_key.is_none() {
            LlmProvider::Nvidia
        } else if openrouter_key.is_some() {
            // Covers both "OpenRouter only" and "both keys set" — OpenRouter
            // is the default for the latter (compat with witness-agents).
            LlmProvider::OpenRouter
        } else {
            return None; // no key → silent fallback
        };

        Some(Self {
            client: reqwest::Client::new(),
            nvidia_api_key: nvidia_key,
            openrouter_api_key: openrouter_key,
            provider,
            tier: tier.to_string(),
        })
    }

    /// Send a chat completion for a specific agent role.
    /// Tries the primary provider first; falls back to OpenRouter on error.
    pub async fn complete_for_role(
        &self,
        role: &str,
        system: &str,
        user: &str,
    ) -> Result<String, String> {
        let (model, max_tokens) = model_for_tier(&self.tier, role);

        match self
            .complete_with_provider(&self.provider, model, system, user, max_tokens)
            .await
        {
            Ok(t) => Ok(t),
            Err(e) if self.provider == LlmProvider::Nvidia && self.openrouter_api_key.is_some() => {
                warn!("[witness-llm] NVIDIA failed ({e}), retrying via OpenRouter");
                self.complete_with_provider(
                    &LlmProvider::OpenRouter,
                    model,
                    system,
                    user,
                    max_tokens,
                )
                .await
            }
            Err(e) => Err(e),
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
    ) -> Result<String, String> {
        let (base_url, api_key) = match provider {
            LlmProvider::Nvidia => (
                NVIDIA_BASE_URL,
                self.nvidia_api_key.as_deref().unwrap_or(""),
            ),
            LlmProvider::OpenRouter => (
                OPENROUTER_BASE_URL,
                self.openrouter_api_key.as_deref().unwrap_or(""),
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
        };

        debug!("[witness-llm] POST {url} model={model} provider={provider:?}");

        let resp = self
            .client
            .post(&url)
            .bearer_auth(api_key)
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

        let parsed: ChatResponse = resp
            .json()
            .await
            .map_err(|e| format!("failed to parse LLM response: {e}"))?;

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
            .map(|s| s.trim().to_string())
            .ok_or_else(|| "LLM returned empty choices".to_string())
    }
}
