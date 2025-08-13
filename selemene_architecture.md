# Selemene Engine - Architecture and Deployment Guide

## System Architecture Overview

### High-Level Architecture
```
┌─────────────────────────────────────────────────────────────────────────┐
│                              Railway.com Cloud                          │
│  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐     │
│  │ Load Balancer   │────│   CDN/Cache     │────│   API Gateway   │     │
│  │ (Railway Proxy) │    │  (Railway Edge) │    │  (Axum Server)  │     │
│  └─────────────────┘    └─────────────────┘    └─────────────────┘     │
│                                                         │               │
│  ┌─────────────────────────────────────────────────────┼─────────────┐ │
│  │                    Application Layer                 │             │ │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌───────▼───────┐     │ │
│  │  │ WebSocket       │  │ Batch Processor │  │ REST API      │     │ │
│  │  │ Service         │  │ Service         │  │ Service       │     │ │
│  │  │ (Real-time)     │  │ (Async Jobs)    │  │ (HTTP/JSON)   │     │ │
│  │  └─────────────────┘  └─────────────────┘  └───────────────┘     │ │
│  └─────────────────┬─────────────────┬─────────────────┬─────────────┘ │
│                    │                 │                 │               │
│  ┌─────────────────▼─────────────────▼─────────────────▼─────────────┐ │
│  │                    Selemene Core Engine                            │ │
│  │  ┌───────────────┐ ┌───────────────┐ ┌───────────────┐ ┌─────────┐ │ │
│  │  │ Calculation   │ │ Hybrid        │ │ Cache         │ │ Config  │ │ │
│  │  │ Orchestrator  │ │ Backend       │ │ Manager       │ │ Manager │ │ │
│  │  └───────────────┘ └───────────────┘ └───────────────┘ └─────────┘ │ │
│  └─────────────────┬─────────────────┬─────────────────┬─────────────┘ │
│                    │                 │                 │               │
│  ┌─────────────────▼─────────────────▼─────────────────▼─────────────┐ │
│  │                    Calculation Engines                             │ │
│  │  ┌───────────────┐ ┌───────────────┐ ┌───────────────┐ ┌─────────┐ │ │
│  │  │ Swiss         │ │ Native Solar  │ │ Native Lunar  │ │ Validation│ │ │
│  │  │ Ephemeris     │ │ Engine        │ │ Engine        │ │ Engine   │ │ │
│  │  │ (Fallback)    │ │ (VSOP87)      │ │ (ELP-2000)    │ │ (Cross-check)│ │
│  │  └───────────────┘ └───────────────┘ └───────────────┘ └─────────┘ │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────────┐ │
│  │                         Data Layer                                  │ │
│  │  ┌───────────────┐ ┌───────────────┐ ┌───────────────┐ ┌─────────┐ │ │
│  │  │ PostgreSQL    │ │ Redis Cache   │ │ Ephemeris     │ │ Config  │ │ │
│  │  │ (Metadata)    │ │ (Hot Data)    │ │ Data Files    │ │ Files   │ │ │
│  │  └───────────────┘ └───────────────┘ └───────────────┘ └─────────┘ │ │
│  └─────────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────┘
```

## Core Engine Architecture

### Selemene Engine Structure
```rust
// Main engine coordinator
pub struct SelemeneEngine {
    // Core calculation engines
    calculation_orchestrator: CalculationOrchestrator,
    
    // Backend selection and routing
    hybrid_backend: HybridBackend,
    
    // Performance optimization
    cache_manager: CacheManager,
    parallel_processor: ParallelProcessor,
    
    // Configuration and monitoring
    config: Arc<RwLock<EngineConfig>>,
    metrics: MetricsCollector,
}

// Hybrid backend system
pub struct HybridBackend {
    swiss_ephemeris: SwissEphemerisEngine,
    native_solar: NativeSolarEngine,
    native_lunar: NativeLunarEngine,
    validation_engine: ValidationEngine,
    
    // Backend selection strategy
    routing_strategy: BackendRoutingStrategy,
    fallback_manager: FallbackManager,
}

// Calculation routing logic
#[derive(Debug, Clone)]
pub enum BackendRoutingStrategy {
    AlwaysNative,           // Use native engines first
    AlwaysSwiss,            // Use Swiss Ephemeris only
    Intelligent,            // Smart routing based on conditions
    Validated,              // Cross-validate results
    PerformanceOptimized,   // Route based on performance needs
}
```

### Calculation Flow
```rust
impl SelemeneEngine {
    pub async fn calculate_panchanga(&self, request: PanchangaRequest) -> Result<PanchangaResult, EngineError> {
        // 1. Request validation and preprocessing
        let validated_request = self.validate_request(request)?;
        
        // 2. Cache lookup
        if let Some(cached_result) = self.cache_manager.get(&validated_request).await? {
            return Ok(cached_result);
        }
        
        // 3. Backend selection
        let backend_choice = self.hybrid_backend.select_backend(&validated_request).await?;
        
        // 4. Calculation execution
        let calculation_result = match backend_choice {
            Backend::Native => self.calculate_with_native(&validated_request).await?,
            Backend::Swiss => self.calculate_with_swiss(&validated_request).await?,
            Backend::Validated => self.calculate_with_validation(&validated_request).await?,
        };
        
        // 5. Result post-processing and caching
        let final_result = self.post_process_result(calculation_result)?;
        self.cache_manager.store(&validated_request, &final_result).await?;
        
        // 6. Metrics collection
        self.metrics.record_calculation(&validated_request, &final_result).await?;
        
        Ok(final_result)
    }
}
```

## Native Engine Implementation

### Solar Engine (VSOP87-based)
```rust
pub struct NativeSolarEngine {
    vsop87_calculator: VSOP87Calculator,
    perturbation_cache: LruCache<JulianDay, SolarPerturbations>,
    coordinate_transformer: CoordinateTransformer,
}

impl NativeSolarEngine {
    /// Calculate solar longitude with high precision
    pub fn solar_longitude(&self, jd: f64, precision: PrecisionLevel) -> Result<f64, SolarEngineError> {
        // Base calculation using VSOP87 theory
        let base_longitude = self.vsop87_calculator.calculate_longitude(jd)?;
        
        // Apply perturbations based on precision requirements
        let perturbations = match precision {
            PrecisionLevel::Standard => self.calculate_major_perturbations(jd)?,
            PrecisionLevel::High => self.calculate_full_perturbations(jd)?,
            PrecisionLevel::Extreme => self.calculate_extended_perturbations(jd)?,
        };
        
        let corrected_longitude = base_longitude + perturbations;
        
        // Normalize to 0-360 degrees
        Ok(corrected_longitude.rem_euclid(360.0))
    }
    
    /// Calculate solar position with velocity
    pub fn solar_position_and_velocity(&self, jd: f64) -> Result<SolarState, SolarEngineError> {
        // Calculate position at three time points for numerical differentiation
        let dt = 1.0 / 86400.0; // 1 second in days
        
        let pos_before = self.solar_longitude(jd - dt, PrecisionLevel::High)?;
        let pos_current = self.solar_longitude(jd, PrecisionLevel::High)?;
        let pos_after = self.solar_longitude(jd + dt, PrecisionLevel::High)?;
        
        // Calculate velocity using central difference
        let velocity = (pos_after - pos_before) / (2.0 * dt);
        
        Ok(SolarState {
            longitude: pos_current,
            longitude_velocity: velocity,
            julian_day: jd,
        })
    }
}
```

### Lunar Engine (ELP-2000 based)
```rust
pub struct NativeLunarEngine {
    elp2000_calculator: ELP2000Calculator,
    perturbation_series: Vec<PerturbationTerm>,
    high_precision_cache: DashMap<u64, LunarState>,
}

impl NativeLunarEngine {
    /// Calculate lunar longitude with ELP-2000 theory
    pub fn lunar_longitude(&self, jd: f64, precision: PrecisionLevel) -> Result<f64, LunarEngineError> {
        // Use appropriate number of terms based on precision
        let max_terms = match precision {
            PrecisionLevel::Standard => 1000,  // Major terms only
            PrecisionLevel::High => 5000,     // Full ELP-2000
            PrecisionLevel::Extreme => 10000, // Extended precision
        };
        
        let lunar_position = self.elp2000_calculator.calculate_position(jd, max_terms)?;
        
        Ok(lunar_position.longitude)
    }
    
    /// Calculate precise Tithi end time using iterative refinement
    pub fn calculate_tithi_end_time(
        &self,
        current_jd: f64,
        target_sun_moon_diff: f64,
        precision: PrecisionLevel
    ) -> Result<f64, LunarEngineError> {
        
        let tolerance = match precision {
            PrecisionLevel::Standard => 1.0 / 1440.0,  // 1 minute
            PrecisionLevel::High => 1.0 / 8640.0,      // 10 seconds
            PrecisionLevel::Extreme => 1.0 / 86400.0,  // 1 second
        };
        
        let mut jd_estimate = current_jd;
        let max_iterations = 20;
        
        for iteration in 0..max_iterations {
            // Calculate current Sun-Moon difference
            let current_diff = self.calculate_sun_moon_difference(jd_estimate)?;
            let error = current_diff - target_sun_moon_diff;
            
            // Check convergence
            if error.abs() < tolerance * 360.0 {
                return Ok(jd_estimate);
            }
            
            // Calculate derivative (rate of change)
            let dt = 1.0 / 86400.0; // 1 second
            let diff_future = self.calculate_sun_moon_difference(jd_estimate + dt)?;
            let derivative = (diff_future - current_diff) / dt;
            
            // Newton-Raphson step
            if derivative.abs() > 1e-10 {
                jd_estimate -= error / derivative;
            } else {
                return Err(LunarEngineError::ConvergenceFailure);
            }
            
            // Prevent unreasonable jumps
            jd_estimate = jd_estimate.clamp(current_jd - 2.0, current_jd + 2.0);
        }
        
        Err(LunarEngineError::MaxIterationsExceeded)
    }
}
```

## Railway.com Deployment Architecture

### Project Structure
```
selemene-engine/
├── Cargo.toml                 # Rust dependencies
├── railway.toml              # Railway configuration
├── Dockerfile                # Multi-stage build
├── docker-compose.yml        # Local development
├── .github/workflows/        # CI/CD pipelines
│   ├── test.yml
│   ├── deploy-staging.yml
│   └── deploy-production.yml
├── src/                      # Rust source code
│   ├── lib.rs               # Library root
│   ├── main.rs              # Binary entry point
│   ├── api/                 # HTTP API layer
│   ├── engines/             # Calculation engines
│   ├── cache/               # Caching system
│   └── config/              # Configuration management
├── data/                     # Astronomical data
│   ├── ephemeris/           # Swiss Ephemeris files
│   ├── constants/           # Calculation constants
│   └── validation/          # Test data
├── tests/                    # Test suites
│   ├── integration/
│   ├── performance/
│   └── validation/
└── docs/                     # Documentation
    ├── api/
    ├── deployment/
    └── cultural-notes/
```

### Railway.com Configuration
```toml
# railway.toml
[build]
builder = "dockerfile"
buildCommand = "cargo build --release"

[deploy]
startCommand = "./target/release/selemene-engine"
healthcheckPath = "/health"
healthcheckTimeout = 30
restartPolicyType = "on_failure"
restartPolicyMaxRetries = 3

# Environment variables
[environments.production]
[environments.production.variables]
RUST_LOG = "info"
ENVIRONMENT = "production"
CACHE_SIZE_MB = "512"
MAX_CONCURRENT_CALCULATIONS = "1000"

[environments.staging]
[environments.staging.variables]
RUST_LOG = "debug"
ENVIRONMENT = "staging"
CACHE_SIZE_MB = "256"
MAX_CONCURRENT_CALCULATIONS = "100"

# Service configuration
[[services]]
name = "selemene-api"
source = "."

[services.variables]
PORT = "8080"
DATABASE_URL = "${{Postgres.DATABASE_URL}}"
REDIS_URL = "${{Redis.REDIS_URL}}"
SWISS_EPHEMERIS_PATH = "/app/data/ephemeris"
NATIVE_ENGINE_ENABLED = "true"
CROSS_VALIDATION_ENABLED = "true"

# Resource allocation
[services.resources]
memoryLimit = "4Gi"
cpuLimit = "2000m"
restartPolicy = "on-failure"

# Horizontal scaling
[services.scaling]
minReplicas = 2
maxReplicas = 10
targetCPUUtilization = 70
targetMemoryUtilization = 80
```

### Multi-Stage Dockerfile
```dockerfile
# Build stage
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    build-essential \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy dependency files
COPY Cargo.toml Cargo.lock ./

# Build dependencies (cached layer)
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy source code
COPY src ./src
COPY data ./data

# Build application
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM ubuntu:22.04

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd --create-home --shell /bin/bash app

WORKDIR /app

# Copy binary and data
COPY --from=builder /app/target/release/selemene-engine ./
COPY --from=builder /app/data ./data

# Set ownership
RUN chown -R app:app /app
USER app

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Start application
CMD ["./selemene-engine"]
```

### Environment-Specific Configurations

#### Production Configuration
```yaml
# config/production.yml
server:
  host: "0.0.0.0"
  port: 8080
  workers: 8

database:
  url: "${DATABASE_URL}"
  max_connections: 20
  timeout: 30

cache:
  redis_url: "${REDIS_URL}"
  size_mb: 512
  ttl_seconds: 3600

calculation:
  default_backend: "intelligent"
  cross_validation_rate: 0.01  # 1% of calculations
  max_concurrent: 1000
  timeout_seconds: 30

engines:
  swiss_ephemeris:
    enabled: true
    data_path: "/app/data/ephemeris"
  native_solar:
    enabled: true
    precision: "high"
  native_lunar:
    enabled: true
    precision: "high"

monitoring:
  metrics_enabled: true
  tracing_enabled: true
  log_level: "info"
```

#### Staging Configuration
```yaml
# config/staging.yml
server:
  host: "0.0.0.0"
  port: 8080
  workers: 4

database:
  url: "${DATABASE_URL}"
  max_connections: 10
  timeout: 30

cache:
  redis_url: "${REDIS_URL}"
  size_mb: 256
  ttl_seconds: 1800

calculation:
  default_backend: "validated"
  cross_validation_rate: 0.1   # 10% validation in staging
  max_concurrent: 100
  timeout_seconds: 60

engines:
  swiss_ephemeris:
    enabled: true
    data_path: "/app/data/ephemeris"
  native_solar:
    enabled: true
    precision: "extreme"  # Test highest precision
  native_lunar:
    enabled: true
    precision: "extreme"

monitoring:
  metrics_enabled: true
  tracing_enabled: true
  log_level: "debug"
```

### CI/CD Pipeline

#### GitHub Actions Workflow
```yaml
# .github/workflows/deploy-production.yml
name: Deploy to Production

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    
    - name: Setup Rust
      uses: dtolnay/rust-toolchain@stable
      
    - name: Setup cache
      uses: Swatinem/rust-cache@v2
      
    - name: Run tests
      run: cargo test --all-features
      
    - name: Run integration tests
      run: cargo test --test integration
      
    - name: Run performance tests
      run: cargo test --test performance --release

  security-audit:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - uses: rustsec/audit-check@v1

  deploy-staging:
    needs: [test, security-audit]
    if: github.event_name == 'pull_request'
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    
    - name: Deploy to Railway Staging
      uses: railwayapp/railway-deploy@v1
      with:
        railway-token: ${{ secrets.RAILWAY_STAGING_TOKEN }}
        service: selemene-staging
        
    - name: Run deployment tests
      run: |
        sleep 60  # Wait for deployment
        curl -f https://selemene-staging.railway.app/health

  deploy-production:
    needs: [test, security-audit]
    if: github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    
    - name: Deploy to Railway Production
      uses: railwayapp/railway-deploy@v1
      with:
        railway-token: ${{ secrets.RAILWAY_PRODUCTION_TOKEN }}
        service: selemene-production
        
    - name: Verify production deployment
      run: |
        sleep 90  # Wait for deployment
        curl -f https://api.selemene.io/health
        curl -f https://api.selemene.io/v1/status
```

## Monitoring and Observability

### Metrics Collection
```rust
use prometheus::{Counter, Histogram, Gauge, Registry};

pub struct EngineMetrics {
    // Request metrics
    pub requests_total: Counter,
    pub request_duration: Histogram,
    pub active_connections: Gauge,
    
    // Calculation metrics
    pub calculations_total: Counter,
    pub calculation_duration: Histogram,
    pub calculation_errors: Counter,
    
    // Backend usage metrics
    pub swiss_ephemeris_usage: Counter,
    pub native_engine_usage: Counter,
    pub cache_hits: Counter,
    pub cache_misses: Counter,
    
    // Accuracy metrics
    pub validation_differences: Histogram,
    pub precision_achieved: Histogram,
}

impl EngineMetrics {
    pub fn record_calculation(&self, backend: &str, duration: f64, accuracy: f64) {
        self.calculations_total.inc();
        self.calculation_duration.observe(duration);
        self.precision_achieved.observe(accuracy);
        
        match backend {
            "swiss" => self.swiss_ephemeris_usage.inc(),
            "native" => self.native_engine_usage.inc(),
            _ => {}
        }
    }
}
```

### Health Check Implementation
```rust
use axum::{Json, response::Json as ResponseJson};
use serde_json::{json, Value};

#[derive(Debug, Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub timestamp: String,
    pub version: String,
    pub components: ComponentHealth,
}

#[derive(Debug, Serialize)]
pub struct ComponentHealth {
    pub database: ComponentStatus,
    pub cache: ComponentStatus,
    pub swiss_ephemeris: ComponentStatus,
    pub native_engines: ComponentStatus,
}

pub async fn health_check(engine: Arc<SelemeneEngine>) -> ResponseJson<Value> {
    let mut status = "healthy";
    let mut components = ComponentHealth::default();
    
    // Check database connectivity
    components.database = match engine.check_database().await {
        Ok(_) => ComponentStatus::healthy(),
        Err(e) => {
            status = "degraded";
            ComponentStatus::unhealthy(e.to_string())
        }
    };
    
    // Check cache connectivity
    components.cache = match engine.check_cache().await {
        Ok(_) => ComponentStatus::healthy(),
        Err(e) => {
            status = "degraded";
            ComponentStatus::unhealthy(e.to_string())
        }
    };
    
    // Check ephemeris data availability
    components.swiss_ephemeris = match engine.check_ephemeris_data().await {
        Ok(_) => ComponentStatus::healthy(),
        Err(e) => {
            status = "degraded";
            ComponentStatus::unhealthy(e.to_string())
        }
    };
    
    // Check native engines
    components.native_engines = match engine.check_native_engines().await {
        Ok(_) => ComponentStatus::healthy(),
        Err(e) => {
            status = "degraded";
            ComponentStatus::unhealthy(e.to_string())
        }
    };
    
    let health = HealthStatus {
        status: status.to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        components,
    };
    
    Json(json!(health))
}
```

## Performance Optimization

### Caching Strategy
```rust
pub struct CacheManager {
    // L1: In-memory hot cache
    l1_cache: Arc<DashMap<CacheKey, CachedResult>>,
    
    // L2: Redis distributed cache
    l2_cache: Arc<redis::Client>,
    
    // L3: Precomputed results
    l3_cache: Arc<PrecomputedCache>,
    
    // Cache statistics
    stats: CacheStats,
}

impl CacheManager {
    pub async fn get(&self, key: &CacheKey) -> Option<CachedResult> {
        // Try L1 cache first
        if let Some(result) = self.l1_cache.get(key) {
            self.stats.l1_hits.inc();
            return Some(result.clone());
        }
        
        // Try L2 cache (Redis)
        if let Ok(data) = self.l2_cache.get::<_, Vec<u8>>(key.to_string()).await {
            self.stats.l2_hits.inc();
            if let Ok(result) = bincode::deserialize(&data) {
                // Populate L1 cache
                self.l1_cache.insert(key.clone(), result.clone());
                return Some(result);
            }
        }
        
        // Try L3 precomputed cache
        if let Some(result) = self.l3_cache.get(key).await {
            self.stats.l3_hits.inc();
            // Populate higher caches
            self.l1_cache.insert(key.clone(), result.clone());
            let _ = self.store_l2(key, &result).await;
            return Some(result);
        }
        
        self.stats.cache_misses.inc();
        None
    }
}
```

### Parallel Processing
```rust
use rayon::prelude::*;
use tokio::task;

impl SelemeneEngine {
    /// Calculate Panchanga for date range in parallel
    pub async fn calculate_range_parallel(
        &self,
        request: RangeRequest,
    ) -> Result<Vec<PanchangaResult>, EngineError> {
        
        let dates = request.generate_dates();
        let chunk_size = (dates.len() / num_cpus::get()).max(1);
        
        // Process in parallel chunks
        let results: Vec<Result<Vec<PanchangaResult>, EngineError>> = 
            stream::iter(dates.chunks(chunk_size))
                .map(|chunk| {
                    let engine = self.clone();
                    let request = request.clone();
                    task::spawn(async move {
                        chunk
                            .iter()
                            .map(|&date| {
                                engine.calculate_panchanga_for_date(date, &request)
                            })
                            .collect::<Result<Vec<_>, _>>()
                    })
                })
                .buffer_unordered(num_cpus::get())
                .try_collect()
                .await?;
        
        // Flatten results
        let flattened: Result<Vec<_>, _> = results
            .into_iter()
            .try_fold(Vec::new(), |mut acc, chunk_result| {
                match chunk_result {
                    Ok(chunk) => {
                        acc.extend(chunk);
                        Ok(acc)
                    }
                    Err(e) => Err(EngineError::ParallelProcessingError(e)),
                }
            });
        
        flattened
    }
}
```

## Security and Compliance

### Authentication and Authorization
```rust
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
    iat: usize,
    tier: String,  // free, premium, enterprise
}

pub async fn authenticate(
    headers: &HeaderMap,
    config: &SecurityConfig,
) -> Result<Claims, AuthError> {
    
    let auth_header = headers
        .get("Authorization")
        .ok_or(AuthError::MissingToken)?
        .to_str()
        .map_err(|_| AuthError::InvalidToken)?;
    
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(AuthError::InvalidToken)?;
    
    let decoding_key = DecodingKey::from_secret(config.jwt_secret.as_ref());
    let validation = Validation::new(Algorithm::HS256);
    
    decode::<Claims>(token, &decoding_key, &validation)
        .map(|token_data| token_data.claims)
        .map_err(|_| AuthError::InvalidToken)
}
```

### Rate Limiting
```rust
use tower::limit::RateLimitLayer;
use tower::ServiceBuilder;

pub fn create_rate_limiter(tier: &str) -> RateLimitLayer {
    let requests_per_minute = match tier {
        "free" => 60,
        "premium" => 1000,
        "enterprise" => 10000,
        _ => 10,
    };
    
    RateLimitLayer::new(
        requests_per_minute,
        Duration::from_secs(60)
    )
}

// Apply rate limiting middleware
let app = Router::new()
    .route("/api/v1/panchanga", post(calculate_panchanga))
    .layer(
        ServiceBuilder::new()
            .layer(rate_limiter)
            .layer(TimeoutLayer::new(Duration::from_secs(30)))
            .layer(TraceLayer::new_for_http())
    );
```

This architecture provides a robust, scalable foundation for the Selemene Engine on Railway.com, with hybrid calculation backends, comprehensive caching, parallel processing, and production-ready monitoring and security features.