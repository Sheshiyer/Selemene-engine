use axum::{
    body::Body,
    http::{header, Request, StatusCode},
};
use std::{
    io::{self, Write},
    sync::{Arc, Mutex, Once, OnceLock},
};
use tower::ServiceExt;
use tracing_subscriber::fmt::MakeWriter;

mod common;

static TRACE_LOG_SLOT: OnceLock<Mutex<Option<Arc<Mutex<Vec<u8>>>>>> = OnceLock::new();
static TRACE_SUBSCRIBER_INIT: Once = Once::new();

struct CaptureWriter;

struct ActiveCaptureWriter;

impl<'a> MakeWriter<'a> for CaptureWriter {
    type Writer = ActiveCaptureWriter;

    fn make_writer(&'a self) -> Self::Writer {
        ActiveCaptureWriter
    }
}

impl Write for ActiveCaptureWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if let Some(slot) = TRACE_LOG_SLOT.get() {
            if let Some(buffer) = slot.lock().unwrap().as_ref() {
                buffer.lock().unwrap().extend_from_slice(buf);
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn ensure_test_subscriber() {
    TRACE_LOG_SLOT.get_or_init(|| Mutex::new(None));

    TRACE_SUBSCRIBER_INIT.call_once(|| {
        let subscriber = tracing_subscriber::fmt()
            .json()
            .with_ansi(false)
            .without_time()
            .with_current_span(true)
            .with_span_list(false)
            .flatten_event(true)
            .with_writer(CaptureWriter)
            .finish();

        let _ = tracing::subscriber::set_global_default(subscriber);
    });
}

async fn capture_logs_for_request<F, Fut, T>(future_factory: F) -> (T, String)
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = T>,
{
    ensure_test_subscriber();

    let buffer = Arc::new(Mutex::new(Vec::new()));
    let slot = TRACE_LOG_SLOT.get().unwrap();
    *slot.lock().unwrap() = Some(buffer.clone());

    let result = future_factory().await;
    tokio::task::yield_now().await;

    *slot.lock().unwrap() = None;
    let logs = String::from_utf8(buffer.lock().unwrap().clone()).unwrap();

    (result, logs)
}

#[serial_test::serial(api_error_trace)]
#[tokio::test]
async fn error_response_trace_id_matches_request_logs() {
    let token = common::generate_test_token(5);
    let router = common::get_router().await.clone();

    let ((status, body), logs) = capture_logs_for_request(|| async move {
        let response = router
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/engines/does-not-exist/calculate")
                    .header(header::AUTHORIZATION, format!("Bearer {}", token))
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        serde_json::to_vec(
                            &serde_json::to_value(common::create_test_birth_input()).unwrap(),
                        )
                        .unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        let status = response.status();
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        (status, body)
    })
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    let trace_id = body["trace_id"]
        .as_str()
        .expect("trace_id should be present in error response");
    assert!(!trace_id.is_empty());
    assert!(
        logs.contains(trace_id),
        "captured logs did not include response trace_id {}\nlogs:\n{}",
        trace_id,
        logs
    );
    assert!(
        logs.contains("/api/v1/engines/does-not-exist/calculate"),
        "captured logs did not include request path\nlogs:\n{}",
        logs
    );
}
