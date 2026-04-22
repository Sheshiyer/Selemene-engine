use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

type ServiceResult<T> = Result<T, String>;

#[derive(Clone)]
pub struct VedicApiService {
    client: Client,
    base_url: String,
    api_key: String,
}

impl VedicApiService {
    pub fn from_env() -> ServiceResult<Self> {
        let api_key = std::env::var("FREE_ASTROLOGY_API_KEY")
            .or_else(|_| std::env::var("VEDIC_API_KEY"))
            .map_err(|_| {
                "Missing Vedic API key. Set FREE_ASTROLOGY_API_KEY or VEDIC_API_KEY.".to_string()
            })?;

        let trimmed_key = api_key.trim();
        if trimmed_key.is_empty() {
            return Err(
                "Vedic API key is empty. Set FREE_ASTROLOGY_API_KEY or VEDIC_API_KEY."
                    .to_string(),
            );
        }

        let base_url = std::env::var("VEDIC_API_BASE_URL")
            .or_else(|_| std::env::var("FREE_ASTROLOGY_API_BASE_URL"))
            .unwrap_or_else(|_| "https://json.freeastrologyapi.com".to_string())
            .trim_end_matches('/')
            .to_string();

        let timeout_secs = std::env::var("VEDIC_API_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(20);

        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .map_err(|e| format!("Failed to build Vedic API client: {e}"))?;

        Ok(Self {
            client,
            base_url,
            api_key: trimmed_key.to_string(),
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn birth_chart(
        &self,
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
        lat: f64,
        lng: f64,
        tzone: f64,
    ) -> ServiceResult<Value> {
        let body = serde_json::json!({
            "year": year,
            "month": month,
            "date": day,
            "hours": hour,
            "minutes": minute,
            "seconds": second,
            "latitude": lat,
            "longitude": lng,
            "timezone": tzone,
            "config": {
                "observation_point": "topocentric",
                "ayanamsha": "lahiri",
                "house_system": "placidus"
            }
        });

        self.post_json("planets", body).await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn navamsa_chart(
        &self,
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        minute: u32,
        second: u32,
        lat: f64,
        lng: f64,
        tzone: f64,
    ) -> ServiceResult<Value> {
        let body = serde_json::json!({
            "year": year,
            "month": month,
            "date": day,
            "hours": hour,
            "minutes": minute,
            "seconds": second,
            "latitude": lat,
            "longitude": lng,
            "timezone": tzone,
            "config": {
                "divisional_chart": "D9",
                "ayanamsha": "lahiri"
            }
        });

        self.post_json("navamsa-chart-info", body).await
    }

    async fn post_json(&self, path: &str, body: Value) -> ServiceResult<Value> {
        let url = format!("{}/{}", self.base_url, path.trim_start_matches('/'));
        let response = self
            .client
            .post(url.clone())
            .header("x-api-key", &self.api_key)
            .header("content-type", "application/json")
            .header("accept", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Failed to call Vedic API endpoint '{path}': {e}"))?;

        let status = response.status();
        if !status.is_success() {
            let error_body = response.text().await.unwrap_or_default();
            return Err(format!(
                "Vedic API endpoint '{path}' returned HTTP {}: {}",
                status.as_u16(),
                error_body
            ));
        }

        response
            .json::<Value>()
            .await
            .map_err(|e| format!("Failed to parse Vedic API endpoint '{path}' response JSON: {e}"))
    }
}