use reqwest::{Client, RequestBuilder};
use serde::de::DeserializeOwned;
use serde_json::Value;
use crate::config::Config;
use crate::error::{Result, WesternApiError};
use crate::types::WesternRequest;

#[derive(Debug, Clone)]
pub struct WesternApiClient {
    config: Config,
    client: Client,
}

impl WesternApiClient {
    pub fn new(config: Config) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self { config, client }
    }

    fn build_request(&self, endpoint: &str, request_data: &WesternRequest) -> RequestBuilder {
        let url = format!("{}/{}", self.config.base_url.trim_end_matches('/'), endpoint.trim_start_matches('/'));
        
        // This specific API uses POST with x-api-key header usually
        self.client.post(&url)
            .header("x-api-key", &self.config.api_key)
            .header("Content-Type", "application/json")
            .json(request_data)
    }

    async fn execute<T: DeserializeOwned>(&self, request_builder: RequestBuilder) -> Result<T> {
        let response = request_builder.send().await?;
        
        if !response.status().is_success() {
             let error_text = response.text().await.unwrap_or_default();
             return Err(WesternApiError::ApiError(format!("API Error: {}", error_text)));
        }

        let data = response.json::<T>().await?;
        Ok(data)
    }

    pub async fn get_western_planets(&self, request: &WesternRequest) -> Result<Value> {
        let builder = self.build_request("western-astrology/planets", request);
        self.execute(builder).await
    }

    pub async fn get_western_houses(&self, request: &WesternRequest) -> Result<Value> {
        let builder = self.build_request("western-astrology/houses", request);
        self.execute(builder).await
    }

    pub async fn get_western_natal_chart(&self, request: &WesternRequest) -> Result<Value> {
        let builder = self.build_request("western-astrology/natal-wheel-chart", request);
        self.execute(builder).await
    }
    
    pub async fn get_western_aspects(&self, request: &WesternRequest) -> Result<Value> {
        let builder = self.build_request("western-astrology/aspects", request);
        self.execute(builder).await
    }

    pub async fn get_geo_details(&self, location: &str) -> Result<Value> {
        let url = format!("{}/geo-location/geo-details", self.config.base_url.trim_end_matches('/'));
        let request_data = serde_json::json!({ "location": location });
        
        let builder = self.client.post(&url)
            .header("x-api-key", &self.config.api_key)
            .header("Content-Type", "application/json")
            .json(&request_data);
            
        self.execute(builder).await
    }

    pub async fn get_timezone_with_dst(&self, latitude: f64, longitude: f64, date: &str) -> Result<Value> {
        let url = format!("{}/time-zone/time-zone-with-dst", self.config.base_url.trim_end_matches('/'));
        let request_data = serde_json::json!({ "latitude": latitude, "longitude": longitude, "date": date });
        
        let builder = self.client.post(&url)
            .header("x-api-key", &self.config.api_key)
            .header("Content-Type", "application/json")
            .json(&request_data);
            
        self.execute(builder).await
    }
}
