//! OpenAI-compatible LLM client for the Witness Dyad.
//!
//! Works with:
//! - OpenAI API (`OPENAI_API_KEY` + default base URL)
//! - NVIDIA NIM Enterprise (`OPENAI_API_KEY` + `WITNESS_LLM_BASE_URL=https://integrate.api.nvidia.com/v1`)
//! - Any OpenAI-compatible endpoint
//!
//! Env vars:
//! - `OPENAI_API_KEY` — required for LLM mode; absent → silent fallback to rule-based dyad
//! - `WITNESS_LLM_BASE_URL` — default `https://api.openai.com/v1`
//! - `WITNESS_LLM_MODEL` — default `gpt-4o`

use serde::{Deserialize, Serialize};
use std::env;
use tracing::{debug, warn};

const DEFAULT_BASE_URL: &str = "https://api.openai.com/v1";
const DEFAULT_MODEL: &str = "gpt-4o";

pub struct LlmClient {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    pub model: String,
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
    message: ChatMessage,
}

impl LlmClient {
    /// Build client from environment. Returns `None` if `OPENAI_API_KEY` is unset.
    pub fn from_env() -> Option<Self> {
        let api_key = env::var("OPENAI_API_KEY").ok()?;
        let base_url = env::var("WITNESS_LLM_BASE_URL")
            .unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());
        let model = env::var("WITNESS_LLM_MODEL")
            .unwrap_or_else(|_| DEFAULT_MODEL.to_string());

        Some(Self {
            client: reqwest::Client::new(),
            api_key,
            base_url,
            model,
        })
    }

    /// Send a chat completion request with a system + user message pair.
    pub async fn complete(
        &self,
        system: &str,
        user: &str,
        max_tokens: u32,
    ) -> Result<String, String> {
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let body = ChatRequest {
            model: self.model.clone(),
            messages: vec![
                ChatMessage { role: "system".into(), content: system.into() },
                ChatMessage { role: "user".into(), content: user.into() },
            ],
            max_tokens,
            temperature: 0.82,
        };

        debug!("[witness-llm] POST {} model={}", url, self.model);

        let resp = self
            .client
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("witness LLM request failed: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            warn!("[witness-llm] error {status}: {text}");
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
            .map(|c| c.message.content.trim().to_string())
            .ok_or_else(|| "LLM returned empty choices".to_string())
    }
}
