# Architecture Overview - Tryambakam Noesis

> **System design and structural patterns** for the 16-engine consciousness computing platform

**Purpose**: Define high-level architecture, layer responsibilities, and integration patterns  
**Scope**: Current Selemene → Target Tryambakam transformation  
**Status**: Transformation phase (late development)

---

## 🎯 Architectural Philosophy

### Three Core Principles

1. **Uniform Interface, Diverse Implementation**  
   All engines implement `ConsciousnessEngine` trait, but internal calculations vary wildly (pure math vs astronomical vs archetypal).

2. **Orchestration, Not Direct Access**  
   All calculations flow through `CalculationOrchestrator`. No direct engine calls. Enables caching, metrics, validation.

3. **Read-Only Wisdom, Mutable Insights**  
   Archetypal data (35 JSON files) is immutable reference. User interpretations/customizations stored separately.

---

## 🏗️ Current Architecture (Selemene Engine)

### Single-Crate Structure

```
selemene-engine/
├── src/
│   ├── api/           # Axum HTTP server
│   │   ├── routes.rs
│   │   ├── handlers.rs
│   │   └── middleware.rs
│   │
│   ├── cache/         # Three-tier caching
│   │   ├── l1_cache.rs      # In-memory LRU (dashmap)
│   │   ├── l2_cache.rs      # Redis distributed
│   │   └── l3_cache.rs      # Disk precomputed
│   │
│   ├── auth/          # JWT + API keys
│   │   └── mod.rs
│   │
│   ├── metrics/       # Prometheus instrumentation
│   │   └── mod.rs
│   │
│   ├── engines/       # Calculation engines
│   │   ├── calculation_orchestrator.rs  # CENTRAL COORDINATOR
│   │   ├── hybrid_backend.rs            # Native vs Swiss routing
│   │   ├── native_solar.rs              # VSOP87 solar engine
│   │   ├── native_lunar.rs              # ELP-2000 lunar engine
│   │   ├── swiss_ephemeris.rs           # Swiss Eph wrapper
│   │   ├── panchanga_calculator.rs      # Vedic Panchanga
│   │   └── validation.rs
│   │
│   ├── time/          # Time systems
│   │   ├── ghati_calculator.rs
│   │   ├── panchanga_integration.rs
│   │   └── realtime_tracker.rs
│   │
│   ├── models/        # Request/response types
│   │   └── mod.rs     # Contains EngineError
│   │
│   ├── config/        # Runtime configuration
│   │   └── mod.rs
│   │
│   ├── main.rs        # Axum server entrypoint
│   └── lib.rs         # Library exports
│
├── data/
│   ├── ephemeris/     # Swiss Ephemeris data files
│   └── wisdom-docs/   # 35 JSON archetypal files (NEW)
│
└── Cargo.toml         # Single crate (will become workspace root)
```

### Key Infrastructure (To Preserve)

**Three-Layer Cache Architecture**
```
Request → Orchestrator
            ↓
         Check L1 (in-memory)?
            ↓ NO
         Check L2 (Redis)?
            ↓ NO
         Check L3 (disk)?
            ↓ NO
         Calculate + Store all tiers
            ↓
         Return result
```

**Hybrid Backend Strategy**
```
Orchestrator receives request
    ↓
Check BackendRoutingStrategy
    ↓
    ├─ Precision = Standard → Native engines (fast)
    ├─ Precision = High → Swiss Ephemeris (accurate)
    ├─ Mode = Validated → Run BOTH, compare results
    └─ Calculation type unsupported by Native → Swiss Ephemeris
```

**Orchestrator Pattern**
```
HTTP Request → API Handler
                    ↓
               CalculationOrchestrator
                    ↓
    ┌───────────────┼───────────────┐
    ↓               ↓               ↓
Cache Lookup   Backend Selection   Metrics
    ↓               ↓               ↓
    └───────────────┴───────────────┘
                    ↓
              Calculation
                    ↓
              Validation
                    ↓
           Store in Cache
                    ↓
             HTTP Response
```

---

## 🔮 Target Architecture (Tryambakam Noesis)

### Cargo Workspace Monorepo

```
selemene-engine/              # Workspace root
├── Cargo.toml                # [workspace] with 17 members
│
├── crates/
│   ├── noesis-core/          # Shared types + trait
│   │   ├── src/
│   │   │   ├── lib.rs        # ConsciousnessEngine trait
│   │   │   ├── types.rs      # EngineInput, EngineOutput, BirthData
│   │   │   ├── error.rs      # EngineError enum
│   │   │   └── ephemeris.rs  # Swiss Ephemeris wrapper (shared)
│   │   └── Cargo.toml
│   │
│   ├── noesis-api/           # Axum HTTP server (from src/api/)
│   │   ├── src/
│   │   │   ├── main.rs       # Server entrypoint
│   │   │   ├── routes.rs     # Engine/workflow routes
│   │   │   ├── handlers.rs   # Request handlers
│   │   │   └── middleware.rs # Auth, logging, metrics
│   │   └── Cargo.toml
│   │
│   ├── noesis-cache/         # Multi-tier cache (from src/cache/)
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── l1.rs         # DashMap LRU
│   │   │   ├── l2.rs         # Redis
│   │   │   └── l3.rs         # Disk
│   │   └── Cargo.toml
│   │
│   ├── noesis-auth/          # Authentication (from src/auth/)
│   │   └── src/lib.rs
│   │
│   ├── noesis-metrics/       # Monitoring (from src/metrics/)
│   │   └── src/lib.rs
│   │
│   ├── noesis-orchestrator/  # Workflow engine
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── calculation_orchestrator.rs  # Refactored from src/engines/
│   │   │   ├── workflow_executor.rs         # Multi-engine workflows
│   │   │   └── backend_router.rs            # Hybrid backend strategy
│   │   └── Cargo.toml
│   │
│   ├── noesis-bridge/        # TypeScript engine adapter
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   └── bridge_engine.rs  # HTTP proxy implementing ConsciousnessEngine
│   │   └── Cargo.toml
│   │
│   ├── noesis-witness/       # Witness prompt generation
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   └── prompt_templates.rs  # Level 0-5 prompts
│   │   └── Cargo.toml
│   │
│   ├── engine-panchanga/     # REFACTORED from src/engines/panchanga_*
│   │   ├── src/
│   │   │   ├── lib.rs        # Implements ConsciousnessEngine
│   │   │   ├── calculator.rs
│   │   │   ├── solar.rs      # From native_solar.rs
│   │   │   └── lunar.rs      # From native_lunar.rs
│   │   └── Cargo.toml
│   │
│   ├── engine-numerology/    # Pure math, no dependencies
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── pythagorean.rs
│   │       └── chaldean.rs
│   │
│   ├── engine-human-design/  # Astronomical, Swiss Eph dependent
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── chart_calculator.rs
│   │       ├── gate_mapping.rs    # Sequential 1-64
│   │       └── data_loader.rs     # Load wisdom-docs/human_design/*.json
│   │
│   ├── engine-biorhythm/     # 3 sine cycles (23/28/33 day)
│   ├── engine-vimshottari/   # 120-year dasha timeline
│   ├── engine-gene-keys/     # Shadow-Gift-Siddhi (HD-dependent)
│   ├── engine-vedic-clock/   # TCM organ clock + Panchanga
│   ├── engine-biofield/      # Biometric analysis (Rust compute core)
│   └── engine-face-reading/  # MediaPipe mesh analysis (Rust compute)
│
├── ts-engines/               # TypeScript engines (Bun HTTP server)
│   ├── package.json          # Bun dependencies
│   ├── server.ts             # HTTP server on port 3001
│   ├── tarot/
│   │   ├── engine.ts         # Implements engine interface
│   │   └── spreads.ts
│   ├── i-ching/
│   │   ├── engine.ts
│   │   └── hexagram_selection.ts
│   ├── enneagram/
│   │   ├── engine.ts
│   │   └── assessment.ts
│   ├── sacred-geometry/
│   │   ├── engine.ts
│   │   └── generators.ts
│   └── sigil-forge/
│       ├── engine.ts
│       └── intent_encoding.ts
│
└── data/
    ├── ephemeris/            # Shared Swiss Ephemeris data
    └── wisdom-docs/          # 35 JSON files (shared across engines)
        ├── human_design/     # 12 files
        ├── astrology/        # 4 files (Vimshottari)
        ├── iching/           # 2 files
        ├── tarot/            # 2 files
        ├── gene_keys/        # 1 file
        ├── enneagram/        # 1 file
        ├── sacred_geometry/  # 2 files
        └── [root level]/     # 11 files (biofield, TCM, face reading)
```

---

## 🧬 Core Trait: ConsciousnessEngine

### The Universal Interface

```rust
// crates/noesis-core/src/lib.rs

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Core trait implemented by all 16 engines (Rust + TypeScript via bridge)
#[async_trait]
pub trait ConsciousnessEngine: Send + Sync {
    /// Unique identifier (e.g., "panchanga", "human-design")
    fn engine_id(&self) -> &str;
    
    /// Human-readable name (e.g., "Vedic Panchanga", "Human Design")
    fn engine_name(&self) -> &str;
    
    /// Minimum consciousness level required to access (0-5)
    /// 0 = Available to all users
    /// 5 = Requires mature self-consciousness practice
    fn required_phase(&self) -> u8;
    
    /// Main calculation method
    /// Takes standard EngineInput, returns standard EngineOutput
    async fn calculate(&self, input: EngineInput) -> Result<EngineOutput, EngineError>;
    
    /// Validate calculation results (optional cross-check)
    async fn validate(&self, output: &EngineOutput) -> Result<ValidationResult, EngineError>;
    
    /// Generate deterministic cache key
    /// SHA-256 of normalized input
    fn cache_key(&self, input: &EngineInput) -> String;
}

/// Standard input container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineInput {
    /// Birth data (optional for some engines like Tarot)
    pub birth_data: Option<BirthData>,
    
    /// Current/query time
    pub current_time: DateTime<Utc>,
    
    /// Geographic coordinates (optional)
    pub location: Option<Coordinates>,
    
    /// Calculation precision level
    pub precision: PrecisionLevel,
    
    /// Engine-specific options
    pub options: HashMap<String, Value>,
}

/// Standard output container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineOutput {
    /// Engine that generated this output
    pub engine_id: String,
    
    /// Engine-specific result as JSON
    pub result: Value,
    
    /// Self-inquiry question (REQUIRED - never empty)
    pub witness_prompt: String,
    
    /// Consciousness level (0-5) this output addresses
    pub consciousness_level: u8,
    
    /// Calculation metadata (time, backend used, etc.)
    pub metadata: CalculationMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BirthData {
    pub date: chrono::NaiveDate,
    pub time: chrono::NaiveTime,
    pub timezone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coordinates {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrecisionLevel {
    Standard,  // Fast, real-time use
    High,      // Increased accuracy
    Extreme,   // Research-grade
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculationMetadata {
    pub calculated_at: DateTime<Utc>,
    pub calculation_time_ms: u64,
    pub backend_used: String,  // "native", "swiss_ephemeris", "validated"
    pub cache_hit: bool,
}

/// Unified error type
#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("Calculation error: {0}")]
    CalculationError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Cache error: {0}")]
    CacheError(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
    
    #[error("Engine unavailable: {0}")]
    EngineUnavailable(String),
}

#[derive(Debug, Serialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub issues: Vec<String>,
}
```

---

## 🔄 Orchestration Layer

### CalculationOrchestrator

```rust
// crates/noesis-orchestrator/src/calculation_orchestrator.rs

pub struct CalculationOrchestrator {
    // Cache tier references
    l1_cache: Arc<L1Cache>,
    l2_cache: Arc<L2Cache>,
    l3_cache: Arc<L3Cache>,
    
    // Backend routing
    backend_router: Arc<BackendRouter>,
    
    // Registered engines
    engines: HashMap<String, Arc<dyn ConsciousnessEngine>>,
    
    // Metrics
    metrics: Arc<MetricsCollector>,
}

impl CalculationOrchestrator {
    /// Execute single-engine calculation with full orchestration
    pub async fn calculate(
        &self,
        engine_id: &str,
        input: EngineInput,
    ) -> Result<EngineOutput, EngineError> {
        let engine = self.engines.get(engine_id)
            .ok_or_else(|| EngineError::EngineUnavailable(engine_id.to_string()))?;
        
        // 1. Generate cache key
        let cache_key = engine.cache_key(&input);
        
        // 2. Check caches (L1 → L2 → L3)
        if let Some(cached) = self.check_caches(&cache_key).await {
            self.metrics.record_cache_hit(engine_id);
            return Ok(cached);
        }
        
        // 3. Select backend (native vs Swiss Ephemeris)
        let backend = self.backend_router.select_backend(&input);
        
        // 4. Execute calculation
        let start = std::time::Instant::now();
        let mut output = engine.calculate(input).await?;
        let elapsed = start.elapsed();
        
        output.metadata.calculation_time_ms = elapsed.as_millis() as u64;
        output.metadata.backend_used = backend.to_string();
        
        // 5. Validate (if configured)
        if self.should_validate(&input) {
            let validation = engine.validate(&output).await?;
            if !validation.valid {
                return Err(EngineError::ValidationError(
                    format!("Validation failed: {:?}", validation.issues)
                ));
            }
        }
        
        // 6. Store in caches
        self.store_in_caches(&cache_key, &output).await;
        
        // 7. Record metrics
        self.metrics.record_calculation(engine_id, elapsed);
        
        Ok(output)
    }
    
    /// Execute multi-engine workflow with parallel execution
    pub async fn execute_workflow(
        &self,
        workflow_id: &str,
        input: EngineInput,
    ) -> Result<WorkflowOutput, EngineError> {
        match workflow_id {
            "birth-blueprint" => {
                // Parallel execution using tokio::join!
                let (numerology, hd, vimshottari) = tokio::join!(
                    self.calculate("numerology", input.clone()),
                    self.calculate("human-design", input.clone()),
                    self.calculate("vimshottari", input.clone()),
                );
                
                // Synthesize results
                Ok(WorkflowOutput {
                    workflow_id: workflow_id.to_string(),
                    engine_outputs: vec![numerology?, hd?, vimshottari?],
                    synthesis: self.synthesize_birth_blueprint(...),
                })
            }
            // ... other workflows
            _ => Err(EngineError::InvalidInput(format!("Unknown workflow: {}", workflow_id))),
        }
    }
}
```

---

## 🌐 API Layer

### Route Structure

```rust
// crates/noesis-api/src/routes.rs

pub fn configure_routes(app: Router, orchestrator: Arc<CalculationOrchestrator>) -> Router {
    app
        // Legacy Selemene endpoints (PRESERVED)
        .route("/panchanga/calculate", post(handlers::legacy_panchanga))
        .route("/ghati/current", get(handlers::legacy_ghati))
        .route("/ghati-panchanga/integration", post(handlers::legacy_ghati_panchanga))
        
        // New v1 engine endpoints
        .route("/api/v1/engines/:engine_id/calculate", post(handlers::calculate_engine))
        .route("/api/v1/engines/:engine_id/info", get(handlers::engine_info))
        .route("/api/v1/engines", get(handlers::list_engines))
        
        // Workflow endpoints
        .route("/api/v1/workflows/:workflow_id/execute", post(handlers::execute_workflow))
        .route("/api/v1/workflows/:workflow_id/info", get(handlers::workflow_info))
        .route("/api/v1/workflows", get(handlers::list_workflows))
        
        // Health & monitoring
        .route("/health", get(handlers::health_check))
        .route("/metrics", get(handlers::prometheus_metrics))
        
        // Middleware
        .layer(middleware::from_fn(auth_middleware))
        .layer(middleware::from_fn(metrics_middleware))
        .layer(middleware::from_fn(cors_middleware))
        .with_state(orchestrator)
}
```

---

## 🔌 TypeScript Bridge Pattern

### Bridge Engine (Rust side)

```rust
// crates/noesis-bridge/src/bridge_engine.rs

pub struct BridgeEngine {
    engine_id: String,
    engine_name: String,
    base_url: String,  // http://localhost:3001
    client: reqwest::Client,
    required_phase: u8,
}

#[async_trait]
impl ConsciousnessEngine for BridgeEngine {
    fn engine_id(&self) -> &str { &self.engine_id }
    fn engine_name(&self) -> &str { &self.engine_name }
    fn required_phase(&self) -> u8 { self.required_phase }
    
    async fn calculate(&self, input: EngineInput) -> Result<EngineOutput, EngineError> {
        // HTTP POST to TypeScript engine
        let response = self.client
            .post(format!("{}/engines/{}/calculate", self.base_url, self.engine_id))
            .json(&input)
            .send()
            .await
            .map_err(|e| EngineError::EngineUnavailable(format!("Bridge error: {}", e)))?;
        
        if !response.status().is_success() {
            return Err(EngineError::CalculationError(
                format!("TypeScript engine returned {}", response.status())
            ));
        }
        
        let output: EngineOutput = response.json().await
            .map_err(|e| EngineError::CalculationError(format!("Parse error: {}", e)))?;
        
        Ok(output)
    }
    
    async fn validate(&self, output: &EngineOutput) -> Result<ValidationResult, EngineError> {
        // Optional: POST to validation endpoint
        Ok(ValidationResult { valid: true, issues: vec![] })
    }
    
    fn cache_key(&self, input: &EngineInput) -> String {
        // Standard SHA-256 deterministic key
        generate_cache_key(&self.engine_id, input)
    }
}
```

### TypeScript Engine (Bun side)

```typescript
// ts-engines/tarot/engine.ts

import type { EngineInput, EngineOutput } from '../types';

export class TarotEngine {
    engineId = 'tarot';
    engineName = 'Tarot Archetypal Reading';
    requiredPhase = 0;
    
    async calculate(input: EngineInput): Promise<EngineOutput> {
        // Load card data
        const deck = await this.loadDeck();
        
        // Perform spread
        const spread = this.performSpread(deck, input);
        
        // Generate witness prompt
        const witnessPrompt = this.generatePrompt(spread, input.options.consciousness_level);
        
        return {
            engine_id: this.engineId,
            result: spread,
            witness_prompt: witnessPrompt,
            consciousness_level: input.options.consciousness_level || 0,
            metadata: {
                calculated_at: new Date().toISOString(),
                calculation_time_ms: 0,
                backend_used: 'typescript',
                cache_hit: false,
            },
        };
    }
}
```

---

## 📊 Data Flow Diagrams

### Single Engine Calculation

```
User Request
    ↓
API Handler (noesis-api)
    ↓
CalculationOrchestrator (noesis-orchestrator)
    ↓
┌───────────────────────────────┐
│ 1. Generate cache key         │
│ 2. Check L1 → L2 → L3        │
│    Cache hit? Return          │
└───────────────────────────────┘
    ↓ NO HIT
┌───────────────────────────────┐
│ 3. Select backend             │
│    (Native vs Swiss Eph)      │
└───────────────────────────────┘
    ↓
┌───────────────────────────────┐
│ 4. Execute engine.calculate() │
│    - Rust engine OR           │
│    - TypeScript via bridge    │
└───────────────────────────────┘
    ↓
┌───────────────────────────────┐
│ 5. Validate (if configured)   │
└───────────────────────────────┘
    ↓
┌───────────────────────────────┐
│ 6. Store in L1/L2/L3         │
│ 7. Record metrics             │
└───────────────────────────────┘
    ↓
API Response to User
```

### Multi-Engine Workflow

```
User Workflow Request
    ↓
API Handler
    ↓
CalculationOrchestrator::execute_workflow()
    ↓
┌─────────────────────────────────────────┐
│ Parallel Execution with tokio::join!    │
│                                          │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐ │
│  │Numerology│  │   HD    │  │Vimshottari│
│  └────┬────┘  └────┬────┘  └────┬────┘ │
│       │            │            │       │
│       └────────────┴────────────┘       │
└─────────────────────────────────────────┘
    ↓
Synthesis Engine
    ↓
WorkflowOutput (combined insights)
    ↓
API Response
```

---

## 🔒 Security Architecture

### Authentication Flow

```
Request
    ↓
Auth Middleware
    ↓
Check Header: Authorization: Bearer <JWT> OR X-API-Key: <key>
    ↓
┌─────────────────────────────────────┐
│ JWT validation:                     │
│ 1. Signature valid?                 │
│ 2. Not expired?                     │
│ 3. Claims include user_id?          │
│                                     │
│ API Key validation:                 │
│ 1. Key exists in database?          │
│ 2. Not revoked?                     │
│ 3. Rate limit not exceeded?         │
└─────────────────────────────────────┘
    ↓ VALID
Attach user context to request
    ↓
Handler executes
```

### Access Control

```
Engine requires consciousness_level = N
    ↓
Check user.consciousness_level >= N
    ↓ YES
Allow access
    ↓ NO
Return 403 Forbidden with message:
"This engine requires consciousness level N. Your current level: M."
```

---

## 📈 Scalability Patterns

### Horizontal Scaling

```
                    Load Balancer
                         ↓
        ┌────────────────┼────────────────┐
        ↓                ↓                ↓
   API Instance 1   API Instance 2   API Instance 3
        ↓                ↓                ↓
        └────────────────┼────────────────┘
                         ↓
                    Redis (L2 Cache)
                         ↓
                    PostgreSQL
```

**Key Points:**
- API instances are stateless
- L1 cache (in-memory) per instance
- L2 cache (Redis) shared across instances
- L3 cache (disk) on shared storage or CDN

### Async Processing

```
User Request (complex workflow)
    ↓
API Handler
    ↓
Create Job in Queue (Redis/RabbitMQ)
    ↓
Return 202 Accepted + job_id
    ↓
User polls: GET /jobs/{job_id}/status

Background Worker:
    ↓
Pull job from queue
    ↓
Execute workflow (long-running)
    ↓
Store result in database
    ↓
Update job status → "completed"
```

---

## 🧪 Testing Strategy

### Testing Pyramid

```
                    /\
                   /  \
                  / E2E \         (Few) - Full API tests
                 /______\
                /        \
               / Integration\     (Some) - Orchestrator + engines
              /____________\
             /              \
            /   Unit Tests   \   (Many) - Individual functions
           /________________\
```

**Unit Tests**: Pure functions, isolated logic  
**Integration Tests**: Orchestrator → Engine → Cache  
**E2E Tests**: HTTP Request → Response validation

---

## 🔧 Configuration Management

### Environment-Based Config

```rust
// crates/noesis-api/src/config.rs

#[derive(Debug, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub cache: CacheConfig,
    pub database: DatabaseConfig,
    pub swiss_ephemeris: SwissEphemerisConfig,
    pub typescript_bridge: BridgeConfig,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Config {
            server: ServerConfig {
                host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("PORT")
                    .unwrap_or_else(|_| "8080".to_string())
                    .parse()?,
            },
            cache: CacheConfig {
                l1_capacity: env::var("L1_CACHE_MB")
                    .unwrap_or_else(|_| "256".to_string())
                    .parse()?,
                redis_url: env::var("REDIS_URL")
                    .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            },
            // ... other configs
        })
    }
}
```

---

## 📝 Summary: Key Architectural Decisions

| Decision | Rationale | Trade-offs |
|----------|-----------|-----------|
| **Uniform ConsciousnessEngine trait** | Enables generic orchestration, consistent API | Less flexibility per engine |
| **Three-tier cache** | Optimize for 85%+ hit rate on birth data | Complexity in invalidation |
| **Orchestrator pattern** | Central control for caching, metrics, validation | Single point of coordination |
| **Cargo workspace** | Modular development, shared types | More complex build setup |
| **TypeScript bridge** | 5 engines stay in TS (inherently visual/interactive) | Network hop, HTTP overhead |
| **Read-only wisdom data** | Preserve archetypal integrity | User customizations need separate storage |

---

**Last Updated**: 2026-01-30  
**Status**: Active transformation from Selemene → Tryambakam  
**Next**: See `.context/migration/transformation-roadmap.md` for implementation phases
