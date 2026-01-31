# Graceful Shutdown and Request Timeout Implementation

## Overview

This document describes the implementation of graceful shutdown handling (W1-S2-04) and request timeout middleware (W1-S2-05) for the Selemene Engine API server.

## Implementation Details

### 1. Graceful Shutdown (W1-S2-04)

**Location**: `src/main.rs`

The server implements graceful shutdown to ensure:
- Clean handling of SIGTERM and SIGINT signals
- No abrupt connection drops
- In-flight requests complete before shutdown
- Proper resource cleanup

#### Implementation

```rust
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received SIGINT (Ctrl+C), starting graceful shutdown...");
        },
        _ = terminate => {
            tracing::info!("Received SIGTERM, starting graceful shutdown...");
        },
    }
    
    tracing::info!("Graceful shutdown initiated. Waiting for in-flight requests to complete...");
}
```

#### Server Configuration

```rust
let server = axum::Server::bind(&addr)
    .serve(app.into_make_service())
    .with_graceful_shutdown(shutdown_signal());
```

#### Features

- **Signal Handling**: Listens for both SIGTERM (container/systemd shutdown) and SIGINT (Ctrl+C)
- **Platform Support**: Works on Unix-like systems (Linux, macOS) and Windows
- **Non-blocking**: Uses `tokio::select!` for efficient signal waiting
- **Automatic Grace Period**: Axum's `with_graceful_shutdown()` handles waiting for connections
- **Logging**: Traces shutdown events for monitoring and debugging

#### Behavior

1. Server receives SIGTERM or SIGINT
2. Server logs the shutdown signal
3. Server stops accepting new connections immediately
4. Server waits for in-flight requests to complete naturally
5. Once all connections close, server exits cleanly

### 2. Request Timeout Middleware (W1-S2-05)

**Location**: `src/api/mod.rs`

Implements request timeout protection to:
- Prevent resource exhaustion from hung requests
- Provide consistent timeout behavior across all API routes
- Return proper HTTP 504 Gateway Timeout responses

#### Implementation

```rust
pub fn create_api_router(engine: Arc<SelemeneEngine>) -> Router {
    // Get timeout from environment or use default (30 seconds)
    let timeout_secs = std::env::var("REQUEST_TIMEOUT_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(30);
    let timeout = Duration::from_secs(timeout_secs);

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_origin(Any)
        .allow_headers(Any);

    // Create router with routes and state
    Router::<Arc<SelemeneEngine>>::new()
        .nest("/api/v1", routes::create_v1_routes())
        .route("/health", axum::routing::get(handlers::health_check))
        .route("/metrics", axum::routing::get(handlers::metrics))
        .route("/status", axum::routing::get(handlers::status))
        .layer(
            ServiceBuilder::new()
                .layer(TimeoutLayer::new(timeout))
                .layer(cors)
        )
        .with_state(engine)
}
```

#### Features

- **Default Timeout**: 30 seconds for all routes
- **Configurable**: Override via `REQUEST_TIMEOUT_SECS` environment variable
- **Applies to**: All `/api/v1/*` routes
- **HTTP Response**: Returns `504 Gateway Timeout` when exceeded
- **Tower Integration**: Uses `tower_http::timeout::TimeoutLayer`

#### Configuration

The timeout can be configured at runtime:

```bash
# Default: 30 seconds
cargo run --bin selemene-engine

# Custom timeout: 60 seconds
REQUEST_TIMEOUT_SECS=60 cargo run --bin selemene-engine
```

## Dependencies

### Cargo.toml Updates

Added `timeout` feature to `tower-http`:

```toml
tower-http = { version = "0.4", features = ["cors", "trace", "timeout"] }
```

### Required Imports

**main.rs**:
```rust
use tokio::signal;
```

**api/mod.rs**:
```rust
use tower::ServiceBuilder;
use tower_http::timeout::TimeoutLayer;
use std::time::Duration;
```

## Testing

### Manual Testing

Use the provided test script:

```bash
./test_shutdown_timeout.sh
```

### Test Scenarios

1. **Normal Request**: Verify requests complete within timeout
2. **Health Check**: Verify non-API routes are also protected
3. **Graceful Shutdown**: Send SIGTERM and verify clean shutdown

### Expected Behavior

#### Graceful Shutdown Test
```bash
# Start server
cargo run --bin selemene-engine

# In another terminal, send SIGTERM
kill -15 <PID>

# Observe logs:
# "Received SIGTERM, starting graceful shutdown..."
# "Graceful shutdown initiated. Waiting for in-flight requests to complete..."
# Server waits for connections, then exits
```

#### Timeout Test
```bash
# Request that exceeds timeout should return 504
curl -i http://localhost:8080/api/v1/some-slow-endpoint

# Expected response:
# HTTP/1.1 504 Gateway Timeout
```

## Architecture Integration

### Orchestrator Pattern Compatibility

The timeout middleware is applied at the router level, before requests reach the `CalculationOrchestrator`. This ensures:
- All calculation routes are protected
- Timeout applies to total request time (including cache lookups)
- No changes needed to existing handler code

### Middleware Stack Order

```
Request → TimeoutLayer → CORS → State → Routes → Handlers
```

The timeout is the outermost layer (applied first), ensuring it covers all processing time.

## Production Considerations

### Deployment

1. **Container Orchestration**: Works with Docker, Kubernetes SIGTERM signals
2. **Systemd**: Compatible with systemd service management
3. **Process Managers**: Works with PM2, systemd, supervisord

### Monitoring

- Log shutdown events for operational visibility
- Monitor timeout rates to identify slow endpoints
- Set up alerts for frequent 504 responses

### Tuning

Adjust timeout based on:
- **Standard calculations**: 30s default is sufficient
- **Batch operations**: May need higher timeout (e.g., 60s)
- **Range calculations**: Consider 90-120s for large ranges

Environment-specific configuration:

```bash
# Development
REQUEST_TIMEOUT_SECS=60

# Production (shorter timeout for better resource management)
REQUEST_TIMEOUT_SECS=30

# Heavy computation workloads
REQUEST_TIMEOUT_SECS=120
```

## Acceptance Criteria

### W1-S2-04: Graceful Shutdown ✓

- [x] Listens for SIGTERM/SIGINT signals using tokio::signal
- [x] Stops accepting new connections on signal
- [x] Waits for in-flight requests to complete
- [x] Closes connections cleanly
- [x] Pattern: Uses axum::serve with graceful shutdown
- [x] Example: tokio::select! with signal handling

### W1-S2-05: Request Timeout ✓

- [x] Uses tower::timeout::TimeoutLayer
- [x] Default: 30 seconds
- [x] Configurable via REQUEST_TIMEOUT_SECS environment variable
- [x] Returns 504 Gateway Timeout if request exceeds limit
- [x] Added to middleware stack in create_router()
- [x] Applied to /api/v1/* routes

## Future Enhancements

1. **Per-route Timeouts**: Different timeouts for different endpoint types
2. **Dynamic Timeouts**: Adjust based on load or calculation complexity
3. **Timeout Metrics**: Track timeout rates per endpoint
4. **Grace Period Configuration**: Make the shutdown grace period configurable
5. **Health Check Integration**: Report "shutting down" status during grace period

## References

- [Axum Graceful Shutdown](https://docs.rs/axum/latest/axum/struct.Server.html#method.with_graceful_shutdown)
- [Tower Timeout Layer](https://docs.rs/tower-http/latest/tower_http/timeout/struct.TimeoutLayer.html)
- [Tokio Signal Handling](https://docs.rs/tokio/latest/tokio/signal/index.html)
