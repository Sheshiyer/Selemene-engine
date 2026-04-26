// Standalone token generator for load testing
// Generates enterprise-tier tokens with high rate limits
//
// Build: rustc --edition 2021 tests/load/generate_token.rs -o tests/load/generate_token
// (Not standalone - uses project's auth service)
//
// Instead, use: cargo run --bin generate_test_credentials
// and modify helpers.js to use multiple user IDs
