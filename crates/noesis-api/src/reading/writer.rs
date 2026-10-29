//! Writing a reading's passes, either with a language model or (without one)
//! by rendering the engine values each pass would be written from.

use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use noesis_witness::llm::LlmClient;
use serde_json::Value;

use super::audit::{audit_section, SectionInput, SectionRubric, SEED_RENDERER};
use super::mode::{ModeDocument, PassPlan};
use super::prompt::{
    max_tokens, prior_tail, render_system_prompt, render_user_prompt, PassContext,
};

/// How much of the reading so far each pass sees, as in the TypeScript orchestrator.
const PRIOR_CHARS: usize = 4000;
const MODEL_ATTEMPTS: usize = 2;

pub enum PassWriter {
    /// Render engine values; no model. Used where a request must answer quickly.
    Seed,
    Model(Arc<LlmClient>),
}

/// The passes to write and, for modes with an authored document, their templates.
pub struct ReadingPlan {
    pub passes: Vec<PassPlan>,
    pub document: Option<&'static ModeDocument>,
}

pub struct ReadingInputs<'a> {
    pub subject_names: &'a [String],
    pub seeds: &'a [(String, Value)],
    pub register: &'a str,
    pub language: Option<&'a str>,
}

#[derive(Clone, Debug)]
pub struct WrittenPass {
    pub id: String,
    pub title: String,
    pub output: String,
    pub rubric: SectionRubric,
}

/// Told before each pass starts, so a job can report progress.
#[async_trait]
pub trait ReadingProgress: Send + Sync {
    async fn pass_started(&self, completed: usize, pass_id: &str);
}

pub struct NoProgress;

#[async_trait]
impl ReadingProgress for NoProgress {
    async fn pass_started(&self, _completed: usize, _pass_id: &str) {}
}

/// Engine values for one pass, without a model.
pub fn render_seed_pass(pass: &PassPlan, seeds: &[(String, Value)]) -> String {
    let mut lines: Vec<String> = vec![format!("Pass {} — {}", pass.id, pass.title)];
    for (id, val) in seeds.iter().take(6) {
        let snippet = serde_json::to_string(val).unwrap_or_default();
        let short = match snippet.char_indices().nth(240) {
            Some((cut, _)) => format!("{}…", &snippet[..cut]),
            None => snippet,
        };
        lines.push(format!("- {id}: {short}"));
    }
    if lines.len() == 1 {
        lines.push("(partial seed context)".to_string());
    }
    if let Some(safeguard) = safeguard_for_pass(&pass.id) {
        lines.push(safeguard.to_string());
    }
    lines.join("\n")
}

fn safeguard_for_pass(pass_id: &str) -> Option<&'static str> {
    match pass_id {
        "wealth" => Some("Witness safeguard: Do not guarantee financial outcomes."),
        "love-marriage" => Some("Witness safeguard: Do not predict marriage inevitability."),
        "health" => Some("Witness safeguard: Do not diagnose."),
        "family-lineage" => Some("Witness safeguard: Do not predict childbirth."),
        _ => None,
    }
}

/// Write every pass in order. Each model pass sees the tail of the passes
/// before it. A model failure (after one retry) fails the whole reading
/// rather than mixing written and seeded sections.
pub async fn write_passes(
    plan: &ReadingPlan,
    inputs: &ReadingInputs<'_>,
    writer: &PassWriter,
    progress: &dyn ReadingProgress,
) -> Result<Vec<WrittenPass>, String> {
    let mut written = Vec::with_capacity(plan.passes.len());
    let mut assembled = String::new();
    for (index, pass) in plan.passes.iter().enumerate() {
        progress.pass_started(index, &pass.id).await;
        let started = Instant::now();
        let (output, model_requested, model_used) = match (writer, plan.document) {
            (PassWriter::Model(llm), Some(doc)) => {
                let ctx = PassContext {
                    subject_names: inputs.subject_names,
                    seeds: inputs.seeds,
                    prior: prior_tail(&assembled, PRIOR_CHARS),
                    register: inputs.register,
                    language: inputs.language,
                };
                let system = render_system_prompt(doc, pass, &ctx);
                let user =
                    render_user_prompt(doc, pass, doc.template(&pass.id, inputs.register)?, &ctx);
                let mut last_error = String::new();
                let mut completion = None;
                for _ in 0..MODEL_ATTEMPTS {
                    match llm
                        .complete_with_budget("synthesis", &system, &user, max_tokens(pass))
                        .await
                    {
                        Ok(c) if !c.text.trim().is_empty() => {
                            completion = Some(c);
                            break;
                        }
                        Ok(_) => last_error = "the model returned an empty section".to_string(),
                        Err(e) => last_error = e,
                    }
                    tracing::warn!("[reading] pass {} failed: {last_error}", pass.id);
                }
                let c = completion
                    .ok_or_else(|| format!("Writing \"{}\" failed: {last_error}", pass.title))?;
                let used = c.served_model.unwrap_or_else(|| c.model.clone());
                (c.text, c.model, used)
            }
            _ => (
                render_seed_pass(pass, inputs.seeds),
                SEED_RENDERER.to_string(),
                SEED_RENDERER.to_string(),
            ),
        };
        let rubric = audit_section(&SectionInput {
            section_id: &pass.id,
            title: &pass.title,
            target_words: pass.target_words,
            output: &output,
            model_requested: &model_requested,
            model_used: &model_used,
            latency_ms: started.elapsed().as_millis() as u64,
            seeds: inputs.seeds,
        });
        assembled.push_str(&format!("\n\n## {}\n\n{}", pass.title, output));
        written.push(WrittenPass {
            id: pass.id.clone(),
            title: pass.title.clone(),
            output,
            rubric,
        });
    }
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reading::mode::l0_mode;
    use axum::{routing::post, Json, Router};
    use serde_json::json;
    use std::sync::Mutex;

    struct Recorder(Mutex<Vec<(usize, String)>>);

    #[async_trait]
    impl ReadingProgress for Recorder {
        async fn pass_started(&self, completed: usize, pass_id: &str) {
            self.0
                .lock()
                .unwrap()
                .push((completed, pass_id.to_string()));
        }
    }

    fn plan() -> ReadingPlan {
        let doc = l0_mode();
        ReadingPlan {
            passes: doc.pass_plan[..2].to_vec(),
            document: Some(doc),
        }
    }

    #[tokio::test]
    async fn seed_writer_renders_engine_values_and_reports_progress() {
        let seeds = vec![("panchanga".to_string(), json!({ "tithi_name": "Dashami" }))];
        let names = vec!["Test Seeker".to_string()];
        let inputs = ReadingInputs {
            subject_names: &names,
            seeds: &seeds,
            register: "l4_l5",
            language: None,
        };
        let recorder = Recorder(Mutex::new(vec![]));
        let passes = write_passes(&plan(), &inputs, &PassWriter::Seed, &recorder)
            .await
            .unwrap();
        assert_eq!(passes.len(), 2);
        assert!(passes[0]
            .output
            .starts_with("Pass opening — Opening\n- panchanga:"));
        assert_eq!(passes[0].rubric.model_used, SEED_RENDERER);
        assert_eq!(
            *recorder.0.lock().unwrap(),
            vec![
                (0, "opening".to_string()),
                (1, "convergence-map".to_string())
            ]
        );
    }

    /// A stand-in OmniRoute that records prompts and answers with the pass id.
    async fn fake_gateway(fail_first: bool) -> (String, Arc<Mutex<Vec<String>>>) {
        let prompts = Arc::new(Mutex::new(Vec::new()));
        let calls = Arc::new(Mutex::new(0usize));
        let (p, c) = (prompts.clone(), calls.clone());
        let app = Router::new().route(
            "/v1/chat/completions",
            post(move |Json(body): Json<Value>| {
                let (p, c) = (p.clone(), c.clone());
                async move {
                    let n = { let mut n = c.lock().unwrap(); *n += 1; *n };
                    let sse = [(axum::http::header::CONTENT_TYPE, "text/event-stream")];
                    if fail_first && n == 1 {
                        return (axum::http::StatusCode::BAD_GATEWAY, sse, "upstream request failed".to_string());
                    }
                    assert_eq!(body["stream"], true, "long-form passes are streamed");
                    let user = body["messages"][1]["content"].as_str().unwrap_or("").to_string();
                    let pass = user.split("Pass: ").nth(1).and_then(|r| r.split('.').next()).unwrap_or("?").to_string();
                    p.lock().unwrap().push(user);
                    let keepalive = json!({ "model": "keepalive", "choices": [{ "delta": {} }] });
                    let chunk = json!({
                        "model": "claude-test",
                        "choices": [{ "delta": { "content": format!("Written {pass} for the Vedic nakshatra.") } }]
                    });
                    (axum::http::StatusCode::OK, sse, format!("data: {keepalive}\n\ndata: {chunk}\n\ndata: [DONE]\n\n"))
                }
            }),
        );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        (format!("http://{addr}/v1"), prompts)
    }

    #[tokio::test]
    async fn model_writer_writes_each_pass_from_its_template_with_prior_context() {
        let (base, prompts) = fake_gateway(true).await;
        let llm = Arc::new(LlmClient::omniroute(
            &base,
            "test-key",
            "noesis-review",
            "standard",
        ));
        let seeds = vec![("panchanga".to_string(), json!({ "tithi_name": "Dashami" }))];
        let names = vec!["Test Seeker".to_string()];
        let inputs = ReadingInputs {
            subject_names: &names,
            seeds: &seeds,
            register: "l4_l5",
            language: None,
        };
        let passes = write_passes(&plan(), &inputs, &PassWriter::Model(llm), &NoProgress)
            .await
            .unwrap();

        assert_eq!(passes[0].output, "Written opening for the Vedic nakshatra.");
        assert_eq!(passes[0].rubric.model_requested, "noesis-review");
        assert_eq!(passes[0].rubric.model_used, "claude-test");
        let prompts = prompts.lock().unwrap();
        assert_eq!(prompts.len(), 2, "the failed first call is retried once");
        assert!(prompts[0].contains("Test Seeker's premium integrated kundali report"));
        assert!(prompts[0].contains("tithi Dashami"));
        assert!(
            prompts[1].contains("## Opening\n\nWritten opening"),
            "the second pass sees the first"
        );
    }

    #[tokio::test]
    async fn a_pass_that_keeps_failing_fails_the_reading() {
        let llm = Arc::new(LlmClient::omniroute(
            "http://127.0.0.1:9/v1",
            "k",
            "m",
            "standard",
        ));
        let names = vec!["Test Seeker".to_string()];
        let inputs = ReadingInputs {
            subject_names: &names,
            seeds: &[],
            register: "l4_l5",
            language: None,
        };
        let error = write_passes(&plan(), &inputs, &PassWriter::Model(llm), &NoProgress)
            .await
            .unwrap_err();
        assert!(error.starts_with("Writing \"Opening\" failed:"), "{error}");
    }
}
