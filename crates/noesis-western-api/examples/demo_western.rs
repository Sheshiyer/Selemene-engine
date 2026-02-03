use noesis_western_api::{WesternApiClient, Config, types::WesternRequest};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load API key from env
    dotenv::dotenv().ok();
    
    let config = Config::from_env().unwrap_or_else(|_| {
        println!("Warning: FREE_ASTROLOGY_API_KEY not set. Using dummy key for dry run build check.");
        Config::new("dummy_key".to_string())
    });

    let client = WesternApiClient::new(config);

    let request = WesternRequest {
        year: 2026,
        month: 2,
        date: 4,
        hours: 12,
        minutes: 0,
        seconds: 0,
        latitude: 40.7128,
        longitude: -74.0060,
        timezone: -5.0,
        config: None,
    };

    println!("Fetching Western Planets...");
    match client.get_western_planets(&request).await {
        Ok(data) => println!("Planets: {}", serde_json::to_string_pretty(&data)?),
        Err(e) => println!("Error fetching planets: {}", e),
    }

    println!("Fetching Western Houses...");
    match client.get_western_houses(&request).await {
        Ok(data) => println!("Houses: {}", serde_json::to_string_pretty(&data)?),
        Err(e) => println!("Error fetching houses: {}", e),
    }

    Ok(())
}
