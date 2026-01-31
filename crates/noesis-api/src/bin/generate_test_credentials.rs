//! Helper program to generate JWT tokens and API keys for testing
//!
//! Usage:
//!   cargo run --bin generate_test_credentials

use noesis_auth::{ApiKey, AuthService};
use chrono::Utc;

#[tokio::main]
async fn main() {
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "noesis-dev-secret-change-in-production".to_string());
    
    let auth_service = AuthService::new(jwt_secret);

    println!("=== Noesis Test Credentials Generator ===\n");

    // Generate JWT tokens for different consciousness levels
    println!("JWT Tokens (24hr expiration):\n");
    
    for level in 0..=5 {
        let token = auth_service
            .generate_jwt_token(
                &format!("test_user_{}", level),
                "premium",
                &vec!["basic:access".to_string(), "panchanga:read".to_string()],
                level,
            )
            .expect("Failed to generate JWT token");
        
        println!("Level {}: {}", level, token);
    }

    println!("\n=== Example Usage ===\n");
    println!("Test with Level 0 (should work for panchanga):");
    let token = auth_service
        .generate_jwt_token("test_user_0", "premium", &vec!["basic:access".to_string()], 0)
        .expect("Failed to generate JWT token");
    
    println!("curl -X POST http://localhost:8080/api/v1/engines/panchanga/calculate \\");
    println!("  -H 'Content-Type: application/json' \\");
    println!("  -H 'Authorization: Bearer {}' \\", token);
    println!("  -d '{{");
    println!("    \"birth_data\": {{");
    println!("      \"name\": \"Test User\",");
    println!("      \"date\": \"1991-08-13\",");
    println!("      \"time\": \"13:31\",");
    println!("      \"latitude\": 12.9629,");
    println!("      \"longitude\": 77.5775,");
    println!("      \"timezone\": \"Asia/Kolkata\"");
    println!("    }}");
    println!("  }}'");
    
    println!("\n=== API Keys ===");
    println!("Note: API keys need to be registered with the AuthService");
    println!("Example API key structure:");
    
    let api_key = ApiKey {
        key: "test-api-key-12345".to_string(),
        user_id: "test_user_api".to_string(),
        tier: "premium".to_string(),
        permissions: vec!["basic:access".to_string(), "panchanga:read".to_string()],
        created_at: Utc::now(),
        expires_at: None,
        last_used: None,
        rate_limit: 1000,
        consciousness_level: 0,
    };
    
    println!("{:#?}", api_key);
    println!("\nTo use API key authentication:");
    println!("curl -X POST http://localhost:8080/api/v1/engines/panchanga/calculate \\");
    println!("  -H 'Content-Type: application/json' \\");
    println!("  -H 'X-API-Key: {}' \\", api_key.key);
    println!("  -d '{{ ... }}'");
}
