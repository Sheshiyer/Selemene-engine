//! HTTP client for FreeAstrologyAPI.com

use chrono::{Datelike, Timelike};
use reqwest::{Client, RequestBuilder, Response, header};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn, error};

use crate::{
    config::Config, 
    error::Result, 
    error::VedicApiError,
    logging,
    panchang::Panchang,
    dasha::{VimshottariDasha, DashaLevel},
    chart::{BirthChart, NavamsaChart},
};

/// HTTP client for FreeAstrologyAPI.com
#[derive(Debug, Clone)]
pub struct VedicApiClient {
    config: Config,
    http_client: Client,
}

impl VedicApiClient {
    /// Create a new API client with the given configuration
    pub fn new(config: Config) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .user_agent(format!(
                "noesis-vedic-api/{}", 
                crate::VERSION
            ))
            .build()
            .expect("Failed to build HTTP client");
        
        info!("VedicApiClient initialized with base_url: {}", config.base_url);
        
        Self {
            config,
            http_client,
        }
    }
    
    /// Create a new API client from environment configuration
    pub fn from_env() -> Result<Self> {
        let config = Config::from_env()?;
        Ok(Self::new(config))
    }
    
    /// Get the configuration
    pub fn config(&self) -> &Config {
        &self.config
    }
    
    /// Build a request with authentication headers
    fn build_request(&self, method: reqwest::Method, path: &str) -> RequestBuilder {
        let url = format!("{}/{}", self.config.base_url.trim_end_matches('/'), path.trim_start_matches('/'));
        
        debug!("Building {} request to {}", method, url);
        
        self.http_client
            .request(method, &url)
            .header(header::ACCEPT, "application/json")
            .header(header::CONTENT_TYPE, "application/json")
            .bearer_auth(&self.config.api_key)
    }
    
    /// Execute a request and handle common errors
    async fn execute_request(&self, request: RequestBuilder) -> Result<Response> {
        let (log_url, log_method) = match request.try_clone().and_then(|req| req.build().ok()) {
            Some(req) => (req.url().to_string(), req.method().clone()),
            None => {
                logging::log_request_build_failure("<unknown>");
                ("<unknown>".to_string(), reqwest::Method::GET)
            }
        };

        logging::log_request(&log_method, &log_url, &self.config.masked_api_key());

        let start = Instant::now();
        let response = match request.send().await {
            Ok(resp) => resp,
            Err(err) => {
                logging::log_error(&log_url, None, start.elapsed(), &err.to_string());
                return Err(err.into());
            }
        };

        let status = response.status();
        debug!("Response status: {}", status);
        
        if status.is_success() {
            logging::log_response(&log_url, status, start.elapsed());
            Ok(response)
        } else {
            // Extract Retry-After header before consuming body
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok());

            let body = response.text().await.unwrap_or_default();
            logging::log_error(&log_url, Some(status), start.elapsed(), &body);
            error!("API error: HTTP {} - {}", status, body);

            match status.as_u16() {
                429 => {
                    warn!(
                        "Rate limited (429). Retry-After: {:?}",
                        retry_after
                    );
                    Err(VedicApiError::RateLimit { retry_after })
                }
                401 => Err(VedicApiError::Configuration {
                    field: "api_key".to_string(),
                    message: "Invalid API key".to_string(),
                }),
                _ => Err(VedicApiError::Api {
                    status_code: status.as_u16(),
                    message: body,
                }),
            }
        }
    }
    
    // ==================== PANCHANG ENDPOINTS ====================
    
    /// Get complete Panchang for a date
    /// 
    /// # Arguments
    /// * `year` - Year (e.g., 1991)
    /// * `month` - Month (1-12)
    /// * `day` - Day (1-31)
    /// * `hour` - Hour (0-23)
    /// * `minute` - Minute (0-59)
    /// * `second` - Second (0-59)
    /// * `lat` - Latitude (-90 to 90)
    /// * `lng` - Longitude (-180 to 180)
    /// * `tzone` - Timezone offset from GMT (e.g., 5.5 for IST)
    pub async fn get_panchang(
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
    ) -> Result<Panchang> {
        info!("Fetching Panchang for {}/{}/{} {}:{}:{}", year, month, day, hour, minute, second);
        
        let params = serde_json::json!({
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
                "ayanamsha": "lahiri"
            }
        });
        
        let request = self.build_request(reqwest::Method::POST, "panchang")
            .json(&params);
        
        let response = self.execute_request(request).await?;
        let panchang: Panchang = response.json().await?;
        
        info!("Panchang retrieved: Tithi={}, Nakshatra={}", 
            panchang.tithi.name(), 
            panchang.nakshatra.name()
        );
        
        Ok(panchang)
    }
    
    /// Get Panchang using chrono::NaiveDateTime
    pub async fn get_panchang_datetime(
        &self,
        datetime: chrono::NaiveDateTime,
        lat: f64,
        lng: f64,
        tz_offset: f64,
    ) -> Result<Panchang> {
        self.get_panchang(
            datetime.year(),
            datetime.month(),
            datetime.day(),
            datetime.hour(),
            datetime.minute(),
            datetime.second(),
            lat,
            lng,
            tz_offset,
        ).await
    }
    
    // ==================== VIMSHOTTARI DASHA ENDPOINTS ====================
    
    /// Get Vimshottari Dasha periods
    /// 
    /// # Arguments
    /// * `year`, `month`, `day` - Birth date
    /// * `hour`, `minute`, `second` - Birth time
    /// * `lat`, `lng` - Birth location
    /// * `tzone` - Timezone offset
    /// * `level` - Dasha depth level (Maha, Antar, Pratyantar, Sookshma)
    pub async fn get_vimshottari_dasha(
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
        level: DashaLevel,
    ) -> Result<VimshottariDasha> {
        info!("Fetching Vimshottari Dasha level: {:?}", level);
        
        let dasha_type = match level {
            DashaLevel::Mahadasha => "maha-dasha",
            DashaLevel::Antardasha => "antar-dasha",
            DashaLevel::Pratyantardasha => "pratyantar-dasha",
            DashaLevel::Sookshma => "sookshma-dasha",
            DashaLevel::Praana => "praana-dasha",
        };
        
        let params = serde_json::json!({
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
                "dasha_type": dasha_type,
                "ayanamsha": "lahiri"
            }
        });
        
        let request = self.build_request(reqwest::Method::POST, "vimshottari-dasha")
            .json(&params);
        
        let response = self.execute_request(request).await?;
        let dasha: VimshottariDasha = response.json().await?;
        
        info!("Vimshottari Dasha retrieved with {} mahadashas", dasha.mahadashas.len());
        
        Ok(dasha)
    }
    
    // ==================== BIRTH CHART ENDPOINTS ====================
    
    /// Get Rashi chart (D1) - main birth chart
    pub async fn get_birth_chart(
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
    ) -> Result<BirthChart> {
        info!("Fetching birth chart for {}/{}/{}", year, month, day);
        
        let params = serde_json::json!({
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
        
        let request = self.build_request(reqwest::Method::POST, "horoscope-chart")
            .json(&params);
        
        let response = self.execute_request(request).await?;
        let chart: BirthChart = response.json().await?;
        
        info!("Birth chart retrieved: Ascendant={}", chart.ascendant.sign.as_str());
        
        Ok(chart)
    }
    
    /// Get Navamsa chart (D9)
    pub async fn get_navamsa_chart(
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
    ) -> Result<NavamsaChart> {
        info!("Fetching Navamsa chart");
        
        let params = serde_json::json!({
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
        
        let request = self.build_request(reqwest::Method::POST, "navamsa-chart")
            .json(&params);
        
        let response = self.execute_request(request).await?;
        let chart: NavamsaChart = response.json().await?;
        
        Ok(chart)
    }
    
    // ==================== UTILITY METHODS ====================
    
    /// Health check - verify API is accessible
    pub async fn health_check(&self) -> Result<bool> {
        debug!("Performing health check");
        
        // Use a simple Panchang request as health check
        let request = self.build_request(reqwest::Method::POST, "panchang")
            .json(&serde_json::json!({
                "year": 2024,
                "month": 1,
                "date": 1,
                "hours": 12,
                "minutes": 0,
                "seconds": 0,
                "latitude": 28.6139,
                "longitude": 77.2090,
                "timezone": 5.5
            }));
        
        match self.execute_request(request).await {
            Ok(_) => {
                info!("Health check passed");
                Ok(true)
            }
            Err(e) => {
                warn!("Health check failed: {}", e);
                Err(e)
            }
        }
    }
    
    /// Get remaining rate limit (if available in headers)
    pub async fn get_rate_limit_status(&self) -> Result<Option<(u64, u64)>> {
        // This would parse rate limit headers if the API provides them
        // For now, return None
        Ok(None)
    }
    
    /// Make a POST request to the given path with JSON body
    pub async fn post<T: serde::de::DeserializeOwned, B: serde::Serialize>(
        &self,
        path: &str,
        body: B,
    ) -> Result<T> {
        let request = self.build_request(reqwest::Method::POST, path)
            .json(&body);
        let response = self.execute_request(request).await?;
        let result: T = response.json().await?;
        Ok(result)
    }
    
    /// Make a GET request to the given path
    pub async fn get<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
    ) -> Result<T> {
        let request = self.build_request(reqwest::Method::GET, path);
        let response = self.execute_request(request).await?;
        let result: T = response.json().await?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let config = Config::new("test_key");
        let client = VedicApiClient::new(config);
        assert_eq!(client.config().api_key, "test_key");
    }

    #[test]
    fn test_masked_api_key() {
        let config = Config::new("sjpRMWCOn340T8JHI8yeL7ucH1741GYT7eMFBMWO");
        let masked = config.masked_api_key();
        assert!(masked.contains("..."));
        assert!(!masked.contains("340T8")); // middle should be hidden
    }
}
