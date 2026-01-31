//! Generate JWT tokens for load testing with enterprise tier (10000 req/min)
//!
//! Usage: cargo run --bin generate_load_test_tokens
//! Output: One JWT token per line for VU indices 0-99

use noesis_auth::AuthService;

fn main() {
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "noesis-dev-secret-change-in-production".to_string());

    let auth_service = AuthService::new(jwt_secret);

    let count: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);

    for i in 0..count {
        let token = auth_service
            .generate_jwt_token(
                &format!("loadtest_user_{}", i),
                "enterprise",                              // 10000 req/min
                &vec!["basic:access".to_string(), "panchanga:read".to_string()],
                5,                                          // max consciousness level
            )
            .expect("Failed to generate JWT token");

        println!("{}", token);
    }
}
