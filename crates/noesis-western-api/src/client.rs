use reqwest::{Client, RequestBuilder};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::Mutex;
use std::time::{Duration, Instant};
// The Cargo.toml has `dashed-map = { package = "dashmap" ... }` so we use the alias
use dashed_map::DashMap;

use crate::config::Config;
use crate::error::{Result, WesternApiError};
use crate::types::WesternRequest;

#[derive(Debug, Clone)]
pub struct WesternApiClient {
    config: Config,
    client: Client,
    cache: Arc<DashMap<String, Value>>,
    last_request: Arc<Mutex<Instant>>,
    daily_requests: Arc<AtomicUsize>,
}

impl WesternApiClient {
    pub fn new(config: Config) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self { 
            config, 
            client,
            cache: Arc::new(DashMap::new()),
            last_request: Arc::new(Mutex::new(Instant::now() - Duration::from_secs(2))), // Initialize in past
            daily_requests: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn build_request(&self, endpoint: &str, request_data: &WesternRequest) -> RequestBuilder {
        let url = format!("{}/{}", self.config.base_url.trim_end_matches('/'), endpoint.trim_start_matches('/'));
        
        self.client.post(&url)
            .header("x-api-key", &self.config.api_key)
            .header("Content-Type", "application/json")
            .json(request_data)
    }

    /// Execute request with caching and rate limiting
    async fn execute_with_policy<T: DeserializeOwned + Serialize + Clone>(&self, builder: RequestBuilder, cache_key: String) -> Result<T> {
        // 1. Check Cache
        if let Some(cached) = self.cache.get(&cache_key) {
            // Deserialize from the cached Value
            let data: T = serde_json::from_value(cached.clone())?;
            return Ok(data);
        }

        // 2. Check Daily Limit
        let count = self.daily_requests.load(Ordering::Relaxed);
        if count >= 50 {
            return Err(WesternApiError::ApiError("Daily limit of 50 requests reached".to_string()));
        }

        // 3. Enforce Rate Limit (1 req/s)
        {
            let mut last = self.last_request.lock().await;
            let elapsed = last.elapsed();
            if elapsed < Duration::from_secs(1) {
                tokio::time::sleep(Duration::from_secs(1) - elapsed).await;
            }
            *last = Instant::now();
        }

        // 4. Execute Network Request
        let response = builder.send().await?;
        
        if !response.status().is_success() {
             let error_text = response.text().await.unwrap_or_default();
             return Err(WesternApiError::ApiError(format!("API Error: {}", error_text)));
        }

        let data = response.json::<T>().await?;

        // 5. Update Cache and Counters
        self.daily_requests.fetch_add(1, Ordering::Relaxed);
        
        // Serialize to Value for generic storage
        let cache_value = serde_json::to_value(data.clone())?;
        self.cache.insert(cache_key, cache_value);

        Ok(data)
    }
    
    // Helper to generate cache keys
    fn generate_key<S: Serialize>(&self, endpoint: &str, request: &S) -> Result<String> {
        let json = serde_json::to_string(request)?;
        Ok(format!("{}:{}", endpoint, json))
    }

    pub async fn get_western_planets(&self, request: &WesternRequest) -> Result<Value> {
        let endpoint = "western-astrology/planets";
        let key = self.generate_key(endpoint, request)?;
        let builder = self.build_request(endpoint, request);
        self.execute_with_policy(builder, key).await
    }

    pub async fn get_western_houses(&self, request: &WesternRequest) -> Result<Value> {
        let endpoint = "western-astrology/houses";
        let key = self.generate_key(endpoint, request)?;
        let builder = self.build_request(endpoint, request);
        self.execute_with_policy(builder, key).await
    }

    pub async fn get_western_natal_chart(&self, request: &WesternRequest) -> Result<Value> {
        let endpoint = "western-astrology/natal-wheel-chart";
        let key = self.generate_key(endpoint, request)?;
        let builder = self.build_request(endpoint, request);
        self.execute_with_policy(builder, key).await
    }

    pub async fn get_western_aspects(&self, request: &WesternRequest) -> Result<Value> {
        let endpoint = "western-astrology/aspects";
        let key = self.generate_key(endpoint, request)?;
        let builder = self.build_request(endpoint, request);
        self.execute_with_policy(builder, key).await
    }

    pub async fn get_geo_details(&self, location: &str) -> Result<Value> {
        let endpoint = "geo-location/geo-details";
        // Create a temporary struct or json for key generation
        let request_data = serde_json::json!({ "location": location });
        let key = self.generate_key(endpoint, &request_data)?;
        
        let url = format!("{}/{}", self.config.base_url.trim_end_matches('/'), endpoint);
        
        let builder = self.client.post(&url)
            .header("x-api-key", &self.config.api_key)
            .header("Content-Type", "application/json")
            .json(&request_data);
            
        self.execute_with_policy(builder, key).await
    }

    pub async fn get_timezone_with_dst(&self, latitude: f64, longitude: f64, date: &str) -> Result<Value> {
        let endpoint = "time-zone/time-zone-with-dst";
        let request_data = serde_json::json!({ "latitude": latitude, "longitude": longitude, "date": date });
        let key = self.generate_key(endpoint, &request_data)?;
        
        let url = format!("{}/{}", self.config.base_url.trim_end_matches('/'), endpoint);
        
        let builder = self.client.post(&url)
            .header("x-api-key", &self.config.api_key)
            .header("Content-Type", "application/json")
            .json(&request_data);
            
        self.execute_with_policy(builder, key).await
    }
}
