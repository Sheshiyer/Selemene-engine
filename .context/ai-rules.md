# AI Rules for Tryambakam Noesis Development

> **Hard constraints** - These are non-negotiable patterns that MUST be followed

**Purpose**: Define absolute rules for AI-assisted development  
**Scope**: All code generation, refactoring, and architectural decisions  
**Authority**: Overrides convenience, overrides "common patterns"

---

## 🚨 CRITICAL: Dual-Paradigm System

This is NOT a typical software project. It requires understanding of:

1. **Technical Precision** (Rust systems programming, async patterns, caching)
2. **Symbolic Depth** (Archetypal meanings, consciousness frameworks)

**You MUST maintain both**. Sacrificing either breaks the system.

---

## 🔒 Architectural Rules (MUST FOLLOW)

### Rule 1: All Calculations Through Orchestrator

**MUST:**
```rust
// CORRECT: Use orchestrator
let result = orchestrator
    .calculate_panchanga(request)
    .await?;
```

**NEVER:**
```rust
// WRONG: Direct engine call bypasses orchestration
let result = panchanga_engine
    .calculate(input)
    .await?;
```

**Why**: The orchestrator coordinates:
- Backend selection (native vs Swiss Ephemeris)
- Cache lookups (L1 → L2 → L3)
- Validation
- Metrics collection
- Error handling

**Bypass means**: Lost caching, lost metrics, inconsistent validation.

---

### Rule 2: ConsciousnessEngine Trait is Contract

**MUST:**
```rust
// All engines implement this trait
impl ConsciousnessEngine for PanchangaEngine {
    fn engine_id(&self) -> &str { "panchanga" }
    fn engine_name(&self) -> &str { "Vedic Panchanga" }
    fn required_phase(&self) -> u8 { 0 }  // Accessible at level 0
    
    async fn calculate(&self, input: EngineInput) -> Result<EngineOutput, EngineError> {
        // Implementation MUST return EngineOutput with witness_prompt
    }
    
    async fn validate(&self, output: &EngineOutput) -> Result<ValidationResult, EngineError> {
        // Validation logic
    }
    
    fn cache_key(&self, input: &EngineInput) -> String {
        // SHA-256 deterministic key
    }
}
```

**NEVER:**
```rust
// WRONG: Custom interface per engine
pub trait PanchangaCalculator {
    async fn calculate_panchanga(&self, date: DateTime) -> PanchangaResult;
}
```

**Why**: Uniform interface enables:
- Generic orchestration
- Consistent caching strategy
- Standardized API routes
- TypeScript bridge pattern

---

### Rule 3: EngineError Enum Only (No anyhow)

**MUST:**
```rust
use crate::models::EngineError;

pub async fn calculate(&self, input: EngineInput) -> Result<EngineOutput, EngineError> {
    let data = load_data()
        .map_err(|e| EngineError::CalculationError(
            format!("Failed to load ephemeris: {}", e)
        ))?;
    
    // ...
}
```

**NEVER:**
```rust
// WRONG: Generic error types
use anyhow::{Result, Context};

pub async fn calculate(&self, input: EngineInput) -> Result<EngineOutput> {
    let data = load_data().context("ephemeris load failed")?;
    // ...
}
```

**Why**: `EngineError` provides:
- Structured error variants (CalculationError, ValidationError, CacheError, etc.)
- API-friendly serialization
- Specific error handling per case
- User-facing error messages

---

### Rule 4: Cache Keys Must Be Deterministic

**MUST:**
```rust
use sha2::{Sha256, Digest};

fn cache_key(&self, input: &EngineInput) -> String {
    let mut hasher = Sha256::new();
    
    // Normalize input before hashing
    hasher.update(input.birth_data.date.to_string());
    hasher.update(input.birth_data.time.to_string());
    hasher.update(format!("{:.4}", input.location.latitude));  // Fixed precision
    hasher.update(format!("{:.4}", input.location.longitude));
    hasher.update(format!("{:?}", input.precision));  // Include precision level
    hasher.update(self.engine_id());  // Include engine ID
    
    format!("{:x}", hasher.finalize())
}
```

**NEVER:**
```rust
// WRONG: Non-deterministic elements
fn cache_key(&self, input: &EngineInput) -> String {
    format!("{:?}_{}", input, Uuid::new_v4())  // UUID makes it non-deterministic
}

// WRONG: Insufficient normalization
fn cache_key(&self, input: &EngineInput) -> String {
    format!("{}{}{}", input.date, input.latitude, input.longitude)  // Float precision issues
}
```

**Why**: Deterministic keys mean:
- Same input = same key = cache hit
- 85%+ cache hit rates for birth data
- Instant results for users
- Cost savings on computation

---

### Rule 5: Every EngineOutput Includes witness_prompt

**MUST:**
```rust
pub struct EngineOutput {
    pub engine_id: String,
    pub result: Value,
    pub witness_prompt: String,  // REQUIRED - never empty
    pub consciousness_level: u8,
    pub metadata: CalculationMetadata,
}

// Example implementation
let output = EngineOutput {
    engine_id: "panchanga".to_string(),
    result: serde_json::to_value(&panchanga_result)?,
    witness_prompt: generate_witness_prompt(&panchanga_result, consciousness_level),
    consciousness_level: 0,
    metadata: CalculationMetadata::new(),
};
```

**NEVER:**
```rust
// WRONG: Missing witness prompt
let output = EngineOutput {
    engine_id: "panchanga".to_string(),
    result: serde_json::to_value(&panchanga_result)?,
    witness_prompt: String::new(),  // Empty is forbidden
    // ...
};

// WRONG: Placeholder text
witness_prompt: "TODO: Add prompt later".to_string(),
```

**Why**: Witness prompts are THE CORE of self-consciousness development:
- Train observer capacity
- Prevent identification with patterns
- Enable authorship vs reactivity

**Without witness prompts**: The system is just data, not consciousness training.

---

## 📊 Data Handling Rules (MUST FOLLOW)

### Rule 6: Wisdom Data is Read-Only

**MUST:**
```rust
// Load once, use many times
lazy_static! {
    static ref GATE_DATA: HashMap<u8, GateInfo> = {
        let json = include_str!("../../data/wisdom-docs/human_design/gates.json");
        serde_json::from_str(json).expect("Failed to parse gates.json")
    };
}

pub fn get_gate(&self, gate_number: u8) -> Option<&GateInfo> {
    GATE_DATA.get(&gate_number)  // Read-only reference
}
```

**NEVER:**
```rust
// WRONG: Runtime modification
fn set_gate_meaning(&mut self, gate: u8, meaning: String) {
    self.gate_data.get_mut(&gate).unwrap().meaning = meaning;
}

// WRONG: Loading on every request
async fn calculate(&self, input: EngineInput) -> Result<EngineOutput, EngineError> {
    let gates = load_json("gates.json")?;  // NO - load once at startup
    // ...
}
```

**Why**:
- Wisdom data represents millennia of crystallized knowledge
- Modifications would corrupt archetypal integrity
- Performance: Load once, use forever
- Consistency: All users get same archetypal data

---

### Rule 7: Preserve Archetypal Depth

**MUST:**
```rust
// Use complete archetypal data from wisdom-docs
#[derive(Debug, Deserialize)]
pub struct GeneKey {
    pub number: u8,
    pub shadow: String,        // Full description
    pub gift: String,          // Full description
    pub siddhi: String,        // Full description
    pub codon_ring: String,    // Biological correlation
    pub amino_acid: String,    // Molecular correlation
    pub keywords: Vec<String>, // Rich symbolic tags
}

// Return full depth to user
let output_json = json!({
    "gene_key": gene_key.number,
    "shadow": gene_key.shadow,
    "gift": gene_key.gift,
    "siddhi": gene_key.siddhi,
    "transformation_path": format!("{} → {} → {}", gene_key.shadow, gene_key.gift, gene_key.siddhi),
    "codon_ring": gene_key.codon_ring,
    "keywords": gene_key.keywords,
});
```

**NEVER:**
```rust
// WRONG: Oversimplified data
#[derive(Debug)]
pub struct GeneKey {
    pub number: u8,
    pub short_description: String,  // Lost the shadow/gift/siddhi depth
}

// WRONG: "Summarizing" archetypal meanings
let summary = format!("Gene Key {} means {}", 
    gene_key.number, 
    "personal growth");  // Lost all nuance
```

**Why**:
- Archetypes are PRECISE symbolic languages
- "Summaries" lose the transformational pathway
- Users need depth to witness patterns accurately
- Shadow-Gift-Siddhi is NOT interchangeable with "good/bad"

---

### Rule 8: Schema Validation for Wisdom Data

**MUST:**
```rust
use serde::{Deserialize, Serialize};

// Define exact schema matching JSON structure
#[derive(Debug, Deserialize, Serialize)]
pub struct HumanDesignGate {
    pub number: u8,
    pub name: String,
    pub keynote: String,
    pub center: String,
    pub circuit: String,
    #[serde(default)]
    pub shadow: Option<String>,
    #[serde(default)]
    pub gift: Option<String>,
    #[serde(default)]
    pub siddhi: Option<String>,
}

// Validate at compile time with type system
fn load_gates() -> Result<HashMap<u8, HumanDesignGate>, serde_json::Error> {
    let json = include_str!("../../data/wisdom-docs/human_design/gates.json");
    serde_json::from_str(json)  // Will fail if schema mismatches
}
```

**NEVER:**
```rust
// WRONG: Untyped JSON access
let gates_value: Value = serde_json::from_str(json)?;
let gate_name = gates_value["1"]["name"].as_str().unwrap();  // Runtime panic risk
```

**Why**:
- Compile-time validation catches schema changes
- Type safety prevents runtime errors
- Clear documentation of expected structure

---

## 🔄 Async/Concurrency Rules (MUST FOLLOW)

### Rule 9: All Public APIs are Async

**MUST:**
```rust
#[async_trait]
pub trait ConsciousnessEngine: Send + Sync {
    async fn calculate(&self, input: EngineInput) -> Result<EngineOutput, EngineError>;
    async fn validate(&self, output: &EngineOutput) -> Result<ValidationResult, EngineError>;
    // Non-async methods allowed for pure functions
    fn cache_key(&self, input: &EngineInput) -> String;
}
```

**NEVER:**
```rust
// WRONG: Blocking I/O in async context
async fn calculate(&self, input: EngineInput) -> Result<EngineOutput, EngineError> {
    let data = std::fs::read_to_string("data.json")?;  // Blocks tokio thread
    // ...
}
```

**Why**:
- Tokio runtime expects non-blocking operations
- Blocking calls starve the executor
- Use `tokio::fs` for file I/O
- Use `tokio::spawn_blocking` for CPU-intensive work

---

### Rule 10: Parallel Workflows Use tokio::join!

**MUST:**
```rust
// Execute multiple engines in parallel
pub async fn execute_workflow(
    &self,
    workflow: WorkflowDefinition,
    input: EngineInput,
) -> Result<WorkflowOutput, EngineError> {
    match workflow.id.as_str() {
        "birth-blueprint" => {
            // Parallel execution
            let (numerology_result, hd_result, vimshottari_result) = tokio::join!(
                self.numerology_engine.calculate(input.clone()),
                self.hd_engine.calculate(input.clone()),
                self.vimshottari_engine.calculate(input.clone()),
            );
            
            // Handle results
            // ...
        }
        _ => Err(EngineError::InvalidWorkflow(workflow.id)),
    }
}
```

**NEVER:**
```rust
// WRONG: Sequential execution when parallel is possible
async fn execute_workflow(&self, workflow: WorkflowDefinition, input: EngineInput) 
    -> Result<WorkflowOutput, EngineError> 
{
    let numerology = self.numerology_engine.calculate(input.clone()).await?;
    let hd = self.hd_engine.calculate(input.clone()).await?;  // Waits for numerology
    let vimshottari = self.vimshottari_engine.calculate(input.clone()).await?;  // Waits for HD
    // Lost parallelism - 3x slower
}
```

**Why**:
- Workflows with independent engines should run in parallel
- User experience: 3 engines in parallel time, not sequential time
- Resource utilization: Use available CPU cores

---

## 🧪 Testing Rules (MUST FOLLOW)

### Rule 11: Integration Tests for Orchestrator

**MUST:**
```rust
// tests/integration/orchestrator_tests.rs
#[tokio::test]
async fn test_panchanga_calculation_end_to_end() {
    let orchestrator = CalculationOrchestrator::new_test();
    
    let request = PanchangaRequest {
        date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        location: Coordinates { latitude: 28.6139, longitude: 77.2090 }, // Delhi
        precision: PrecisionLevel::Standard,
    };
    
    let result = orchestrator.calculate_panchanga(request).await;
    
    assert!(result.is_ok());
    let panchanga = result.unwrap();
    assert!(!panchanga.tithi.name.is_empty());
    assert!(!panchanga.nakshatra.name.is_empty());
}
```

**NEVER:**
```rust
// WRONG: Only unit tests (miss integration issues)
#[test]
fn test_tithi_calculation() {
    let tithi = calculate_tithi_pure(123.45);
    assert_eq!(tithi, 5);
    // Doesn't test orchestrator, caching, error handling
}
```

**Why**:
- Orchestrator coordinates multiple systems
- Integration tests catch boundary issues
- End-to-end validation ensures user-facing APIs work

---

### Rule 12: Accuracy Tests Against Reference Data

**MUST:**
```rust
// tests/validation/accuracy_tests.rs
#[tokio::test]
async fn test_human_design_accuracy() {
    let engine = HumanDesignEngine::new();
    
    // Known accurate chart from professional software
    let reference = ReferenceChart {
        birth_date: NaiveDate::from_ymd_opt(1990, 1, 15).unwrap(),
        birth_time: NaiveTime::from_hms_opt(14, 30, 0).unwrap(),
        location: Coordinates { latitude: 40.7128, longitude: -74.0060 },
        expected_type: "Generator",
        expected_authority: "Sacral",
        expected_profile: "6/2",
        expected_personality_sun: (17, 3),  // Gate 17, Line 3
        expected_design_sun: (18, 6),       // Gate 18, Line 6
    };
    
    let result = engine.calculate(reference.into()).await.unwrap();
    let chart: HumanDesignChart = serde_json::from_value(result.result).unwrap();
    
    assert_eq!(chart.type_name, reference.expected_type);
    assert_eq!(chart.authority, reference.expected_authority);
    assert_eq!(chart.profile, reference.expected_profile);
    assert_eq!(chart.personality_sun.gate, reference.expected_personality_sun.0);
    assert_eq!(chart.personality_sun.line, reference.expected_personality_sun.1);
}
```

**Why**:
- Accuracy is philosophical necessity for trustworthy mirrors
- Cross-validation against professional software
- Prevents regressions in astronomical calculations

---

## 🌐 API Rules (MUST FOLLOW)

### Rule 13: Preserve Legacy Endpoints

**MUST:**
```rust
// All existing Panchanga/Ghati routes MUST continue working
pub fn configure_routes(app: Router) -> Router {
    app
        // Legacy endpoints (preserved)
        .route("/panchanga/calculate", post(handlers::calculate_panchanga))
        .route("/ghati/current", get(handlers::get_current_ghati))
        .route("/ghati-panchanga/integration", post(handlers::ghati_panchanga_integration))
        
        // New engine/workflow endpoints
        .route("/api/v1/engines/:engine_id/calculate", post(handlers::calculate_engine))
        .route("/api/v1/workflows/:workflow_id/execute", post(handlers::execute_workflow))
}
```

**NEVER:**
```rust
// WRONG: Breaking existing endpoints
pub fn configure_routes(app: Router) -> Router {
    app
        .route("/api/v1/engines/panchanga/calculate", post(handlers::calculate_panchanga))
        // Old /panchanga/calculate is gone - breaks existing clients
}
```

**Why**:
- Selemene has existing users/integrations
- Transformation should be additive, not destructive
- Version routes separately (v1, v2) for breaking changes

---

### Rule 14: Consistent Error Response Format

**MUST:**
```rust
#[derive(Serialize)]
pub struct ErrorResponse {
    pub error_code: String,
    pub message: String,
    pub details: Option<Value>,
}

impl From<EngineError> for ErrorResponse {
    fn from(error: EngineError) -> Self {
        match error {
            EngineError::CalculationError(msg) => ErrorResponse {
                error_code: "CALCULATION_ERROR".to_string(),
                message: msg,
                details: None,
            },
            EngineError::ValidationError(msg) => ErrorResponse {
                error_code: "VALIDATION_ERROR".to_string(),
                message: msg,
                details: None,
            },
            // ... other variants
        }
    }
}
```

**NEVER:**
```rust
// WRONG: Inconsistent error responses
async fn calculate_handler(input: Json<Input>) -> Result<Json<Output>, String> {
    let result = orchestrator.calculate(input.0).await
        .map_err(|e| format!("Error: {}", e))?;  // Just a string - not structured
    Ok(Json(result))
}
```

**Why**:
- Clients need structured errors to handle programmatically
- Consistent format across all endpoints
- Distinguishable error types (validation vs calculation vs cache)

---

## 🚫 Anti-Patterns (NEVER DO)

### Never #1: Bypass the Orchestrator

```rust
// FORBIDDEN
let solar_engine = NativeSolarEngine::new();
let position = solar_engine.calculate_position(jd).await?;
```

**Always**: Go through `CalculationOrchestrator`.

### Never #2: Use anyhow in Public APIs

```rust
// FORBIDDEN
pub async fn calculate(&self, input: EngineInput) -> anyhow::Result<EngineOutput>
```

**Always**: Use `Result<T, EngineError>`.

### Never #3: Empty Witness Prompts

```rust
// FORBIDDEN
witness_prompt: String::new()
witness_prompt: "".to_string()
```

**Always**: Generate meaningful self-inquiry question.

### Never #4: Modify Wisdom Data at Runtime

```rust
// FORBIDDEN
self.gate_data.insert(gate_number, modified_gate);
```

**Always**: Wisdom data is immutable reference material.

### Never #5: Non-Deterministic Cache Keys

```rust
// FORBIDDEN
format!("{}_{}_{}", input, Uuid::new_v4(), SystemTime::now())
```

**Always**: SHA-256 of normalized input only.

---

## ✅ Summary Checklist

Before submitting any code, verify:

- [ ] All calculations go through `CalculationOrchestrator`
- [ ] All engines implement `ConsciousnessEngine` trait
- [ ] All errors use `EngineError` enum (no `anyhow`)
- [ ] All cache keys are SHA-256 deterministic
- [ ] All `EngineOutput` includes non-empty `witness_prompt`
- [ ] Wisdom data is read-only (no runtime modifications)
- [ ] Archetypal depth is preserved (no oversimplification)
- [ ] All public methods are `async fn`
- [ ] Parallel workflows use `tokio::join!`
- [ ] Integration tests cover orchestrator paths
- [ ] Accuracy tests validate against reference data
- [ ] Legacy API endpoints still work
- [ ] Error responses use consistent structure

---

**Last Updated**: 2026-01-30  
**Authority**: Overrides all other guidance  
**Violation Handling**: Immediate refactor required
