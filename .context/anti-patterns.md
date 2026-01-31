# Anti-Patterns for Tryambakam Noesis

> **What NOT to do** - Common mistakes and forbidden approaches

**Purpose**: Document patterns that will break the system  
**For**: AI assistants, developers, code reviewers  
**Authority**: Violations require immediate refactor

---

## 🚫 Architectural Anti-Patterns

### ❌ Direct Engine Access (Bypassing Orchestrator)

**The Mistake:**
```rust
// WRONG - Calling engine directly
let panchanga_engine = PanchangaEngine::new();
let result = panchanga_engine.calculate(input).await?;
```

**Why It's Broken:**
- Loses cache lookups (L1/L2/L3 all bypassed)
- No metrics collection
- No backend strategy (native vs Swiss Ephemeris)
- No validation pipeline
- Inconsistent error handling

**The Correct Way:**
```rust
// CORRECT - Through orchestrator
let result = orchestrator
    .calculate_panchanga(request)
    .await?;
```

**When You'll Be Tempted:**
- "Just testing the engine quickly"
- "Don't need caching for this one call"
- "Want to benchmark raw engine performance"

**Resist**: Even tests should use orchestrator (use `new_test()` variant if needed).

---

### ❌ Creating Custom Engine Interfaces

**The Mistake:**
```rust
// WRONG - Engine-specific trait
pub trait PanchangaCalculator {
    async fn calculate_panchanga(&self, date: DateTime) -> PanchangaResult;
    async fn calculate_tithi(&self, jd: f64) -> Tithi;
}

pub trait NumerologyCalculator {
    async fn calculate_life_path(&self, date: NaiveDate) -> u8;
}
```

**Why It's Broken:**
- Cannot use generic orchestration
- Cannot route through standard API
- Cannot apply consistent caching
- Cannot generate consistent responses
- TypeScript bridge won't work

**The Correct Way:**
```rust
// CORRECT - Uniform ConsciousnessEngine trait
impl ConsciousnessEngine for PanchangaEngine {
    async fn calculate(&self, input: EngineInput) -> Result<EngineOutput, EngineError> {
        // All engines use same interface
    }
}
```

---

### ❌ Synchronous Swiss Ephemeris Initialization

**The Mistake:**
```rust
// WRONG - Blocking in async context
impl CalculationOrchestrator {
    pub fn new(config: EngineConfig) -> Self {
        let swiss_eph = swisseph::Handler::new();  // Blocks tokio thread
        swiss_eph.set_ephe_path(&config.swiss_ephemeris_path);  // Blocks
        
        Self { swiss_eph, /* ... */ }
    }
}
```

**Why It's Broken:**
- Blocks async executor
- Starves tokio worker threads
- Poor startup performance
- Can cause deadlocks under load

**The Current Reality:**
This is a **known limitation** documented in `selemene_architecture.md`. It exists but needs refactoring.

**The Future Correct Way:**
```rust
// CORRECT - Async initialization
impl CalculationOrchestrator {
    pub async fn new(config: EngineConfig) -> Self {
        let swiss_eph = tokio::spawn_blocking(move || {
            let mut handler = swisseph::Handler::new();
            handler.set_ephe_path(&config.swiss_ephemeris_path);
            handler
        }).await.expect("Swiss Ephemeris init failed");
        
        Self { swiss_eph, /* ... */ }
    }
}
```

---

### ❌ Adding Features Without Cache Invalidation Strategy

**The Mistake:**
```rust
// Adding new calculation feature
pub async fn calculate_advanced_panchanga(&self, input: AdvancedInput) 
    -> Result<AdvancedOutput, EngineError> 
{
    // New feature implementation
    // ... but no cache key strategy defined
    // ... and no invalidation logic
}
```

**Why It's Broken:**
- Users get stale results after you change the algorithm
- No way to clear old cache entries
- Cache grows indefinitely
- Different cache key formats across features

**The Correct Way:**
```rust
// 1. Define cache key strategy
fn cache_key(&self, input: &AdvancedInput) -> String {
    let mut hasher = Sha256::new();
    hasher.update(format!("v2_advanced_"));  // Version prefix
    hasher.update(/* normalized input */);
    format!("{:x}", hasher.finalize())
}

// 2. Document invalidation strategy in code
/// Cache invalidation: 30-day TTL for birth data
/// Version prefix changed if algorithm changes
/// Clear with: redis-cli --scan --pattern "v2_advanced_*" | xargs redis-cli del
```

---

## 🗄️ Data Handling Anti-Patterns

### ❌ Modifying Wisdom Data at Runtime

**The Mistake:**
```rust
// WRONG - Mutating archetypal data
impl HumanDesignEngine {
    pub fn update_gate_meaning(&mut self, gate: u8, new_meaning: String) {
        self.gate_data.get_mut(&gate).unwrap().meaning = new_meaning;
    }
    
    pub fn add_custom_center(&mut self, center: CustomCenter) {
        self.center_data.insert(center.id, center);
    }
}
```

**Why It's Broken:**
- Corrupts archetypal integrity
- Different users get different meanings
- Lost reference to source wisdom
- Cannot rollback changes
- Violates read-only contract

**The Correct Way:**
```rust
// CORRECT - Read-only access
lazy_static! {
    static ref GATE_DATA: HashMap<u8, GateInfo> = load_gates();
}

impl HumanDesignEngine {
    pub fn get_gate(&self, gate: u8) -> Option<&GateInfo> {
        GATE_DATA.get(&gate)  // Immutable reference
    }
}
```

**If You Need Customization:**
- Add user-specific interpretations as SEPARATE layer
- Never modify source wisdom data
- Store customizations in user database, not archetypal corpus

---

### ❌ Oversimplifying Archetypal Meanings

**The Mistake:**
```rust
// WRONG - Collapsed complexity
#[derive(Serialize)]
pub struct GeneKeySimplified {
    pub number: u8,
    pub summary: String,  // "This key is about personal growth"
}

// WRONG - Lost transformation pathway
pub fn get_gene_key_meaning(&self, number: u8) -> String {
    format!("Gene Key {} represents {}", number, "positive change")
}
```

**Why It's Broken:**
- Shadow-Gift-Siddhi is PRECISE transformation pathway
- "Growth" doesn't distinguish Shadow from Gift
- Lost consciousness evolution framework
- Users can't witness specific patterns
- Reduced to platitude ("be positive")

**The Correct Way:**
```rust
// CORRECT - Full archetypal depth
#[derive(Serialize)]
pub struct GeneKey {
    pub number: u8,
    pub shadow: String,    // Unconscious frequency
    pub gift: String,      // Present creative frequency
    pub siddhi: String,    // Transcendent frequency
    pub codon_ring: String,
    pub keywords: Vec<String>,
}

// Return complete transformation map
pub fn get_gene_key(&self, number: u8) -> &GeneKey {
    &self.gene_keys[&number]  // All fields intact
}
```

---

### ❌ Loading JSON on Every Request

**The Mistake:**
```rust
// WRONG - Repeated file I/O
async fn calculate(&self, input: EngineInput) -> Result<EngineOutput, EngineError> {
    let gates_json = tokio::fs::read_to_string("data/wisdom-docs/human_design/gates.json").await?;
    let gates: HashMap<u8, Gate> = serde_json::from_str(&gates_json)?;
    
    // Use gates...
}
```

**Why It's Broken:**
- File I/O on every request (slow)
- Repeated JSON parsing (CPU waste)
- No error handling if file disappears
- Doesn't scale past single-digit QPS

**The Correct Way:**
```rust
// CORRECT - Load once at startup
lazy_static! {
    static ref GATES: HashMap<u8, Gate> = {
        let json = include_str!("../../data/wisdom-docs/human_design/gates.json");
        serde_json::from_str(json).expect("Invalid gates.json schema")
    };
}

async fn calculate(&self, input: EngineInput) -> Result<EngineOutput, EngineError> {
    let gate = GATES.get(&input.gate_number).ok_or(EngineError::InvalidInput)?;
    // Use gate...
}
```

---

### ❌ Untyped JSON Access

**The Mistake:**
```rust
// WRONG - Runtime JSON navigation
let gates_value: Value = serde_json::from_str(gates_json)?;
let gate_name = gates_value["gates"]["1"]["name"]
    .as_str()
    .ok_or("Missing gate name")?;
let center = gates_value["gates"]["1"]["center"]
    .as_str()
    .unwrap();  // Panic if schema changed
```

**Why It's Broken:**
- No compile-time validation
- Runtime panics if schema changes
- Typos in field names not caught
- Hard to refactor
- No IDE autocomplete

**The Correct Way:**
```rust
// CORRECT - Typed structs
#[derive(Deserialize)]
pub struct GateData {
    pub gates: HashMap<String, Gate>,
}

#[derive(Deserialize)]
pub struct Gate {
    pub number: u8,
    pub name: String,
    pub center: String,
    pub keynote: String,
}

let gate_data: GateData = serde_json::from_str(gates_json)?;
let gate = &gate_data.gates["1"];
let center = &gate.center;  // Type-safe access
```

---

## 🔄 Async/Concurrency Anti-Patterns

### ❌ Blocking I/O in Async Context

**The Mistake:**
```rust
// WRONG - std::fs blocks tokio
async fn load_ephemeris(&self, date: NaiveDate) -> Result<EphemerisData, EngineError> {
    let data = std::fs::read_to_string(format!("data/ephe/{}.dat", date))?;
    Ok(parse_ephemeris(data))
}

// WRONG - Thread::sleep blocks tokio
async fn rate_limit(&self) {
    std::thread::sleep(Duration::from_secs(1));  // Blocks worker thread
}
```

**Why It's Broken:**
- Blocks tokio worker threads
- Other tasks can't make progress
- Reduced concurrency
- Poor scalability

**The Correct Way:**
```rust
// CORRECT - tokio::fs
async fn load_ephemeris(&self, date: NaiveDate) -> Result<EphemerisData, EngineError> {
    let data = tokio::fs::read_to_string(format!("data/ephe/{}.dat", date)).await?;
    Ok(parse_ephemeris(data))
}

// CORRECT - tokio::time::sleep
async fn rate_limit(&self) {
    tokio::time::sleep(Duration::from_secs(1)).await;
}
```

---

### ❌ Sequential Execution of Independent Tasks

**The Mistake:**
```rust
// WRONG - Sequential when parallel is possible
async fn execute_birth_blueprint(&self, input: EngineInput) 
    -> Result<WorkflowOutput, EngineError> 
{
    let numerology = self.numerology.calculate(input.clone()).await?;
    let hd = self.human_design.calculate(input.clone()).await?;  // Waits unnecessarily
    let vimshottari = self.vimshottari.calculate(input.clone()).await?;  // Waits unnecessarily
    
    Ok(synthesize(numerology, hd, vimshottari))
}
```

**Why It's Broken:**
- 3 independent calculations
- Each waits for previous to complete
- 3x slower than necessary
- Poor resource utilization

**The Correct Way:**
```rust
// CORRECT - Parallel execution
async fn execute_birth_blueprint(&self, input: EngineInput) 
    -> Result<WorkflowOutput, EngineError> 
{
    let (numerology, hd, vimshottari) = tokio::join!(
        self.numerology.calculate(input.clone()),
        self.human_design.calculate(input.clone()),
        self.vimshottari.calculate(input.clone()),
    );
    
    Ok(synthesize(numerology?, hd?, vimshottari?))
}
```

---

### ❌ Spawning Unbounded Tasks

**The Mistake:**
```rust
// WRONG - No limit on concurrent tasks
async fn process_batch(&self, requests: Vec<Request>) {
    for request in requests {
        tokio::spawn(async move {
            self.process(request).await
        });
    }
    // No wait, no error handling, no limit
}
```

**Why It's Broken:**
- Memory exhaustion with large batches
- No backpressure
- Lost error results
- System overload

**The Correct Way:**
```rust
// CORRECT - Bounded concurrency
use futures::stream::{self, StreamExt};

async fn process_batch(&self, requests: Vec<Request>) -> Vec<Result<Output, EngineError>> {
    stream::iter(requests)
        .map(|req| self.process(req))
        .buffer_unordered(10)  // Max 10 concurrent
        .collect()
        .await
}
```

---

## 🧪 Testing Anti-Patterns

### ❌ Only Unit Tests (No Integration)

**The Mistake:**
```rust
// WRONG - Testing individual functions in isolation
#[test]
fn test_tithi_calculation() {
    assert_eq!(calculate_tithi(123.45), 5);
}

#[test]
fn test_nakshatra_calculation() {
    assert_eq!(calculate_nakshatra(234.56), 12);
}

// Missing: End-to-end orchestrator tests
```

**Why It's Broken:**
- Doesn't test component integration
- Doesn't test cache behavior
- Doesn't test error propagation
- Doesn't test API contracts
- Real bugs slip through

**The Correct Way:**
```rust
// CORRECT - Both unit AND integration tests

// Unit test
#[test]
fn test_tithi_calculation() {
    assert_eq!(calculate_tithi(123.45), 5);
}

// Integration test
#[tokio::test]
async fn test_panchanga_end_to_end() {
    let orchestrator = CalculationOrchestrator::new_test();
    let request = PanchangaRequest { /* ... */ };
    let result = orchestrator.calculate_panchanga(request).await;
    assert!(result.is_ok());
}
```

---

### ❌ No Reference Data Validation

**The Mistake:**
```rust
// WRONG - Just checking it doesn't crash
#[tokio::test]
async fn test_human_design() {
    let engine = HumanDesignEngine::new();
    let input = test_input();
    let result = engine.calculate(input).await;
    assert!(result.is_ok());  // But is it CORRECT?
}
```

**Why It's Broken:**
- Algorithm could be completely wrong
- Calculations could drift over time
- Astronomical accuracy not validated
- Users get wrong charts

**The Correct Way:**
```rust
// CORRECT - Validate against known accurate charts
#[tokio::test]
async fn test_human_design_accuracy() {
    let engine = HumanDesignEngine::new();
    
    // Reference from professional software
    let reference = ReferenceChart {
        birth_date: date(1990, 1, 15),
        expected_personality_sun: (17, 3),  // Gate 17, Line 3
        expected_design_sun: (18, 6),
        // ... all expected values
    };
    
    let result = engine.calculate(reference.clone().into()).await.unwrap();
    let chart: HDChart = serde_json::from_value(result.result).unwrap();
    
    assert_eq!(chart.personality_sun.gate, 17);
    assert_eq!(chart.personality_sun.line, 3);
    // ... validate all fields
}
```

---

## 🌐 API Anti-Patterns

### ❌ Breaking Legacy Endpoints

**The Mistake:**
```rust
// WRONG - Removing existing routes during transformation
pub fn configure_routes(app: Router) -> Router {
    app
        // Old routes deleted:
        // .route("/panchanga/calculate", ...) -- GONE
        
        // Only new routes:
        .route("/api/v1/engines/panchanga/calculate", post(new_handler))
}
```

**Why It's Broken:**
- Existing integrations break
- Users lose access
- No migration path
- Violates semantic versioning

**The Correct Way:**
```rust
// CORRECT - Preserve old, add new
pub fn configure_routes(app: Router) -> Router {
    app
        // Legacy routes (preserved)
        .route("/panchanga/calculate", post(handlers::legacy_panchanga))
        .route("/ghati/current", get(handlers::legacy_ghati))
        
        // New v1 routes
        .route("/api/v1/engines/panchanga/calculate", post(handlers::panchanga_v1))
}
```

---

### ❌ Inconsistent Error Responses

**The Mistake:**
```rust
// WRONG - Different error formats per endpoint
async fn handler_a(input: Json<Input>) -> Result<Json<Output>, (StatusCode, String)> {
    Err((StatusCode::BAD_REQUEST, "Invalid input".to_string()))
}

async fn handler_b(input: Json<Input>) -> Result<Json<Output>, String> {
    Err(format!("Error: {}", "something"))
}

async fn handler_c(input: Json<Input>) -> Result<Json<Output>, Json<CustomError>> {
    Err(Json(CustomError { msg: "failed" }))
}
```

**Why It's Broken:**
- Clients can't parse errors consistently
- No error codes for programmatic handling
- Some errors return JSON, some return text
- Poor API experience

**The Correct Way:**
```rust
// CORRECT - Uniform error structure
#[derive(Serialize)]
pub struct ErrorResponse {
    pub error_code: String,
    pub message: String,
    pub details: Option<Value>,
}

// All handlers use same type
async fn handler(input: Json<Input>) -> Result<Json<Output>, (StatusCode, Json<ErrorResponse>)> {
    Err((
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            error_code: "INVALID_INPUT".to_string(),
            message: "Birth date is required".to_string(),
            details: None,
        })
    ))
}
```

---

## 🎯 Consciousness-Specific Anti-Patterns

### ❌ Empty or Placeholder Witness Prompts

**The Mistake:**
```rust
// WRONG - No witness prompt
let output = EngineOutput {
    engine_id: "panchanga".to_string(),
    result: result_json,
    witness_prompt: String::new(),  // Empty
    consciousness_level: 0,
    metadata: CalculationMetadata::new(),
};

// WRONG - Placeholder
witness_prompt: "TODO: Add later".to_string(),

// WRONG - Generic non-specific
witness_prompt: "Reflect on this information.".to_string(),
```

**Why It's Broken:**
- **Witness prompts ARE the self-consciousness training mechanism**
- Without them, system is just data delivery
- Generic prompts don't trigger pattern recognition
- Lost the entire point of Noesis

**The Correct Way:**
```rust
// CORRECT - Specific self-inquiry question
let witness_prompt = match consciousness_level {
    0 => format!(
        "Notice: Today's nakshatra is {}. When have you felt {} energy in your life before?",
        panchanga.nakshatra.name,
        panchanga.nakshatra.quality
    ),
    1 => format!(
        "Observe: The {} nakshatra repeats every 27 days. Do you notice patterns in your energy when it returns?",
        panchanga.nakshatra.name
    ),
    2 => format!(
        "Witness: Your birth nakshatra is {} ({}), and today's is {} ({}). How do these two energies interact in you?",
        birth_nakshatra.name, birth_nakshatra.quality,
        current_nakshatra.name, current_nakshatra.quality
    ),
    // ... level-specific prompts
};
```

---

### ❌ Prescriptive Language (Telling Instead of Mirroring)

**The Mistake:**
```rust
// WRONG - Prescriptive
witness_prompt: "You should meditate during this nakshatra.".to_string(),
witness_prompt: "This is a good time for creative work.".to_string(),
witness_prompt: "Avoid important decisions today.".to_string(),
```

**Why It's Broken:**
- Removes user agency
- System becomes authority, not mirror
- Prevents self-authorship
- Creates dependency

**The Correct Way:**
```rust
// CORRECT - Inquiry-based mirroring
witness_prompt: "This nakshatra is traditionally associated with creativity. Do you notice creative impulses today?".to_string(),
witness_prompt: "What decisions feel aligned with your inner authority right now?".to_string(),
witness_prompt: "Observe: What energy are you bringing to today's activities?".to_string(),
```

---

## 🔍 Common Temptations

### "Just This Once" Patterns

❌ "Just this once, I'll call the engine directly (faster for testing)"  
✅ Use orchestrator with `new_test()` config

❌ "Just this once, I'll use `anyhow` (quicker to write)"  
✅ Use `EngineError` - structured errors are worth it

❌ "Just this once, I'll leave witness prompt empty (will add later)"  
✅ Add it now - "later" never comes

❌ "Just this once, I'll simplify the archetypal data (easier to work with)"  
✅ Preserve depth - users need it

---

## ✅ Summary: What Makes a Pattern "Anti"?

A pattern is an anti-pattern if it:

1. **Breaks the orchestrator coordination** (lost caching, metrics, validation)
2. **Loses archetypal depth** (oversimplified wisdom data)
3. **Removes witness prompts** (lost self-consciousness training)
4. **Violates type safety** (untyped JSON, `anyhow` errors)
5. **Blocks async execution** (std::fs, thread::sleep in async)
6. **Breaks existing APIs** (removes legacy endpoints)
7. **Creates non-deterministic cache keys** (UUIDs, timestamps)
8. **Modifies wisdom data** (mutates read-only archetypes)

---

**Last Updated**: 2026-01-30  
**When You Violate These**: Refactor immediately - technical debt compounds fast
