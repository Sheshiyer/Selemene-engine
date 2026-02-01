# Noesis Workflows

Multi-engine workflow orchestration for the Tryambakam consciousness engine platform.

## Overview

Workflows combine multiple consciousness engines to provide synthesized, multi-perspective insights. Each workflow is designed for a specific type of inquiry, coordinating engines that complement each other to reveal patterns and themes that wouldn't be visible from any single system alone.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    WorkflowOrchestrator                         │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │   Engine    │  │   Engine    │  │   Engine    │  ...        │
│  │  Registry   │  │   Cache     │  │   Bridge    │             │
│  └─────────────┘  └─────────────┘  └─────────────┘             │
├─────────────────────────────────────────────────────────────────┤
│                    Parallel Execution                           │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐   │
│  │Engine 1 │ │Engine 2 │ │Engine 3 │ │Engine 4 │ │Engine N │   │
│  └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘   │
│       │           │           │           │           │         │
│       └───────────┴───────────┴───────────┴───────────┘         │
│                           │                                     │
│                    ┌──────┴──────┐                              │
│                    │  Synthesis  │                              │
│                    └─────────────┘                              │
└─────────────────────────────────────────────────────────────────┘
```

## Available Workflows

### 1. Birth Blueprint (`birth-blueprint`)

**Engines:** Numerology, Human Design, Gene Keys

**Purpose:** Comprehensive natal analysis revealing your core identity patterns

**Use When:** You want to understand your fundamental nature and innate patterns based on birth data

**Synthesis:** Cross-references life path number with HD type and Gene Keys activation sequence to find consistent identity themes

**TTL:** 24 hours (natal data is fixed)

### 2. Daily Practice (`daily-practice`)

**Engines:** Panchanga, Vedic Clock, Biorhythm

**Purpose:** Daily rhythm and awareness guidance

**Use When:** Planning your day, understanding current energetic conditions

**Synthesis:** Aligns Vedic time quality with biorhythmic cycles to identify optimal timing windows

**TTL:** 1 hour (time-sensitive)

### 3. Decision Support (`decision-support`)

**Engines:** Tarot, I-Ching, Human Design (Authority)

**Purpose:** Multi-system perspective for important decisions

**Use When:** Facing a significant choice and wanting diverse perspectives

**Synthesis:** Compares archetypal guidance (Tarot/I-Ching) with your decision-making strategy (HD Authority)

**TTL:** 15 minutes (question-specific)

### 4. Self-Inquiry (`self-inquiry`)

**Engines:** Gene Keys, Enneagram

**Purpose:** Deep psychological and spiritual self-exploration

**Use When:** Engaged in shadow work or wanting to understand reactive patterns

**Synthesis:** Maps Gene Keys shadows onto Enneagram fixations to reveal growth edges

**TTL:** 24 hours (core patterns are stable)

### 5. Creative Expression (`creative-expression`)

**Engines:** Sigil Forge, Sacred Geometry

**Purpose:** Generate visual and symbolic content for intention work

**Use When:** Creating sacred art, setting intentions, or ritual work

**Synthesis:** Combines sigil encoding with geometric harmonics

**TTL:** 15 minutes (intention-specific)

### 6. Full Spectrum (`full-spectrum`)

**Engines:** ALL 14 engines in parallel

**Purpose:** Complete self-portrait across all consciousness systems

**Use When:** Seeking comprehensive overview, beginning major life transitions

**Synthesis:** Identifies themes appearing in 3+ engines across 5 categories

**TTL:** 1 hour

## Engine Categories

Engines are organized into five categories for synthesis:

| Category | Engines | Nature |
|----------|---------|--------|
| **Natal** | Human Design, Gene Keys, Numerology, Enneagram | Fixed patterns from birth |
| **Temporal** | Panchanga, Vedic Clock, Biorhythm, Vimshottari | Time-based cycles |
| **Archetypal** | Tarot, I-Ching | Symbolic guidance |
| **Somatic** | Biofield, Face Reading | Body-based patterns |
| **Creative** | Sacred Geometry, Sigil Forge | Generative/visual |

## Synthesis Patterns

### Theme Detection Algorithm

1. **Extract** keywords and concepts from each engine result
2. **Normalize** terms to common vocabulary (e.g., "leader" → "leadership")
3. **Count** occurrences across engines
4. **Rank** by frequency and categorize

**Primary Themes:** Appear in 3+ engines (high confidence)
**Secondary Themes:** Appear in 2 engines (notable but less certain)

### Theme Categories

| Category | Focus | Example Sources |
|----------|-------|-----------------|
| **Identity** | Who you are | HD Type, Life Path, Enneagram |
| **Timing** | When to act | Panchanga tithi, Biorhythm peaks |
| **Shadow** | What to witness | Gene Keys shadow, Enneagram fear |
| **Gift** | What to cultivate | Gene Keys gift, HD channels |
| **Direction** | Where to go | Tarot advice, I-Ching hexagram |

### Alignment vs Tension

**Alignments** occur when multiple engines point in similar directions. These are synthesized as reinforcing patterns:
> "Your HD Manifestor type aligns with your Life Path 1 leadership and the Gene Keys gift of Initiative."

**Tensions** are framed as "multiple perspectives" rather than contradictions:
> "Multiple perspectives emerge: your Enneagram 9 seeks peace while your Gene Keys shadow of Conflict asks for examination. What does this polarity reveal?"

## Caching Strategy

| Workflow Type | TTL | Rationale |
|--------------|-----|-----------|
| **Natal** | 24 hours | Birth data never changes |
| **Temporal** | 1 hour | Time-sensitive but not instant |
| **Archetypal** | 15 minutes | Question/intention specific |
| **Full Spectrum** | 1 hour | Balance of natal and temporal |

### Cache Key Structure

```rust
WorkflowCacheKey {
    workflow_id: "birth-blueprint",
    input_hash: 0xABCD1234,      // Hash of birth date, location, question
    engine_versions: "v1.0.0",   // Invalidates on engine updates
}
```

### Invalidation

- **By Workflow:** `cache.invalidate_workflow("birth-blueprint")`
- **By Engine:** `cache.invalidate_engine("numerology")` — invalidates all workflows using that engine

## Performance Targets

| Metric | Target | Measured |
|--------|--------|----------|
| Full Spectrum (14 engines) | <2 seconds | ~50ms (parallel) |
| Single Workflow (3 engines) | <500ms | ~15ms (parallel) |
| Cache Hit | <1ms | <0.5ms |

### Parallel Execution

All engines execute concurrently via `futures::join_all`:

```rust
let futures = engines.iter().map(|e| e.calculate(input.clone()));
let results = join_all(futures).await;
```

Execution time ≈ max(engine_times), not sum(engine_times).

## Adding New Workflows

### 1. Define the Workflow

```rust
WorkflowDefinition {
    id: "my-workflow".into(),
    name: "My Workflow".into(),
    description: "Description of purpose".into(),
    engine_ids: vec!["engine1".into(), "engine2".into()],
}
```

### 2. Register with Orchestrator

```rust
orchestrator.register_workflow(my_workflow);
```

### 3. Implement Synthesis (Optional)

For custom synthesis logic beyond theme detection:

```rust
impl Synthesizer for MyWorkflowSynthesizer {
    fn synthesize(&self, outputs: &HashMap<String, EngineOutput>) -> SynthesisResult {
        // Custom logic here
    }
}
```

### 4. Set Caching TTL

```rust
WorkflowTtl::Custom(Duration::from_secs(1800)) // 30 minutes
```

## API Usage

### Execute a Workflow

```rust
let orchestrator = WorkflowOrchestrator::new();
orchestrator.register_engine(Arc::new(NumerologyEngine::new()));
// ... register more engines

let input = EngineInput {
    birth_data: Some(BirthData { ... }),
    current_time: Utc::now(),
    ...
};

let result = orchestrator.execute_workflow("birth-blueprint", input, user_phase).await?;

for (engine_id, output) in result.engine_outputs {
    println!("{}: {}", engine_id, output.witness_prompt);
}
```

### Execute Full Spectrum with Synthesis

```rust
let workflow = FullSpectrumWorkflow::new(engines);
let result = workflow.execute(input).await?;

let synthesizer = FullSpectrumSynthesizer::new();
let synthesis = synthesizer.synthesize(&result);

for theme in synthesis.primary_themes {
    println!("Theme: {} ({}% strength)", theme.theme, theme.strength * 100.0);
    if let Some(prompt) = theme.witness_prompt {
        println!("  Inquiry: {}", prompt);
    }
}
```

### Execute Specific Categories

```rust
let result = workflow.execute_categories(input, &[
    EngineCategory::Natal,
    EngineCategory::Temporal,
]).await?;
```

## Error Handling

Workflows handle engine failures gracefully:

- Individual engine failures don't fail the entire workflow
- Failed engines are logged and excluded from results
- `failed_engines` map contains error details
- Synthesis adjusts confidence based on available data

```rust
let result = workflow.execute(input).await?;

if !result.failed_engines.is_empty() {
    for (engine, error) in &result.failed_engines {
        eprintln!("Engine {} failed: {}", engine, error);
    }
}

// Workflow still succeeds with partial results
assert!(result.engines_succeeded > 0);
```

## Phase Gating

Engines may require minimum consciousness phase levels:

```rust
// Engine requires phase 3
orchestrator.register_engine(Arc::new(AdvancedEngine::new())); // phase: 3

// User at phase 1 cannot access
let result = orchestrator.execute_workflow("workflow", input, 1).await?;
// AdvancedEngine excluded from results
```

Phase levels:
- 0: Public access
- 1-2: Basic engagement
- 3-4: Advanced practice
- 5: Full access

## Testing

Run integration tests:
```bash
cargo test --package noesis-orchestrator
```

Run benchmarks:
```bash
cargo bench --package noesis-orchestrator
```

Verify full spectrum performance:
```bash
cargo bench --package noesis-orchestrator -- full_spectrum
```

---

## Synthesis Deep Dive

### Cross-Engine Theme Detection

The synthesis layer identifies patterns appearing across multiple engines:

#### Algorithm

```rust
fn synthesize(outputs: &HashMap<String, EngineOutput>) -> SynthesisResult {
    // 1. Extract keywords from each engine
    let keywords = outputs.iter()
        .flat_map(|(engine_id, output)| {
            extract_keywords(&output.result)
        })
        .collect::<Vec<_>>();
    
    // 2. Normalize to common vocabulary
    let normalized = keywords.iter()
        .map(|k| normalize_keyword(k))
        .collect::<Vec<_>>();
    
    // 3. Count occurrences
    let counts = count_occurrences(&normalized);
    
    // 4. Classify themes
    let primary = counts.iter()
        .filter(|(_, count)| **count >= 3)
        .collect();
    let secondary = counts.iter()
        .filter(|(_, count)| **count == 2)
        .collect();
    
    SynthesisResult { primary, secondary, ... }
}
```

#### Theme Categories

| Category | Description | Sources |
|----------|-------------|---------|
| **Identity** | Core self patterns | HD Type, Life Path, Enneagram |
| **Timing** | Optimal action windows | Panchanga, Biorhythm, Vimshottari |
| **Shadow** | Unconscious patterns | Gene Keys Shadow, Enneagram Fixation |
| **Gift** | Conscious strengths | Gene Keys Gift, HD Channels |
| **Direction** | Guidance for path | Tarot, I-Ching, HD Authority |

### Workflow-Specific Synthesizers

#### Birth Blueprint Synthesizer

Identifies natal patterns across Numerology, Human Design, and Gene Keys:

```rust
pub struct BirthBlueprintSynthesizer;

impl Synthesizer for BirthBlueprintSynthesizer {
    fn synthesize(&self, outputs: &HashMap<String, EngineOutput>) -> SynthesisResult {
        // Extract Life Path from numerology
        let life_path = extract_life_path(&outputs["numerology"]);
        
        // Extract Type from Human Design  
        let hd_type = extract_hd_type(&outputs["human-design"]);
        
        // Extract Life's Work from Gene Keys
        let lifes_work = extract_lifes_work(&outputs["gene-keys"]);
        
        // Find alignments
        let alignments = find_identity_alignments(life_path, hd_type, lifes_work);
        
        // Generate synthesis narrative
        SynthesisResult {
            primary_themes: alignments,
            narrative: generate_birth_narrative(alignments),
            witness_prompt: generate_birth_prompt(alignments),
        }
    }
}
```

#### Daily Practice Synthesizer

Aligns temporal patterns across Panchanga, Vedic Clock, and Biorhythm:

```rust
pub struct DailyPracticeSynthesizer;

impl Synthesizer for DailyPracticeSynthesizer {
    fn synthesize(&self, outputs: &HashMap<String, EngineOutput>) -> SynthesisResult {
        // Extract tithi quality from Panchanga
        let tithi = extract_tithi(&outputs["panchanga"]);
        
        // Extract dosha period from Vedic Clock
        let dosha = extract_current_dosha(&outputs["vedic-clock"]);
        
        // Extract biorhythm levels
        let rhythms = extract_biorhythm(&outputs["biorhythm"]);
        
        // Find optimal timing windows
        let windows = find_optimal_windows(tithi, dosha, rhythms);
        
        SynthesisResult {
            temporal_windows: windows,
            recommendations: generate_daily_recommendations(windows),
            witness_prompt: generate_daily_prompt(tithi, dosha, rhythms),
        }
    }
}
```

### Alignment vs Tension Analysis

#### Alignments

When multiple engines point in similar directions:

```json
{
  "type": "alignment",
  "theme": "Leadership",
  "sources": ["HD Manifestor", "Life Path 1", "Gene Key 7 Gift"],
  "strength": 0.85,
  "narrative": "Multiple systems indicate natural leadership capacity",
  "witness_prompt": "With leadership appearing across your charts, how do you already lead without trying?"
}
```

#### Tensions

When engines present seemingly contradictory information:

```json
{
  "type": "tension",
  "theme": "Action vs Stillness",
  "sources": ["HD Wait to Respond", "Biorhythm Physical High", "Panchanga Active Tithi"],
  "narrative": "Multiple perspectives emerge: inner strategy says wait while outer conditions favor action",
  "witness_prompt": "When your body says 'go' but your design says 'wait,' what wisdom lives in that tension?"
}
```

### Full Spectrum Synthesis

The `FullSpectrumSynthesizer` processes all 14 engines:

```rust
pub struct FullSpectrumSynthesizer;

impl FullSpectrumSynthesizer {
    pub fn synthesize(&self, result: &FullSpectrumResult) -> SynthesisResult {
        // 1. Categorize engine outputs
        let by_category = categorize_outputs(&result.engine_outputs);
        
        // 2. Extract themes from each category
        let natal_themes = extract_natal_themes(&by_category[EngineCategory::Natal]);
        let temporal_themes = extract_temporal_themes(&by_category[EngineCategory::Temporal]);
        let archetypal_themes = extract_archetypal_themes(&by_category[EngineCategory::Archetypal]);
        let somatic_themes = extract_somatic_themes(&by_category[EngineCategory::Somatic]);
        let creative_themes = extract_creative_themes(&by_category[EngineCategory::Creative]);
        
        // 3. Find cross-category themes
        let all_themes = vec![
            natal_themes, temporal_themes, archetypal_themes, 
            somatic_themes, creative_themes
        ];
        let cross_themes = find_cross_category_themes(&all_themes);
        
        // 4. Generate comprehensive synthesis
        SynthesisResult {
            primary_themes: cross_themes.filter(|t| t.occurrences >= 3),
            secondary_themes: cross_themes.filter(|t| t.occurrences == 2),
            category_summaries: generate_category_summaries(&by_category),
            alignments: find_alignments(&cross_themes),
            tensions: find_tensions(&cross_themes),
            witness_prompt: generate_full_spectrum_prompt(&cross_themes),
        }
    }
}
```

### Witness Prompt Generation

Synthesis generates a unified witness prompt from multiple engines:

```rust
fn generate_synthesis_prompt(themes: &[CrossEngineTheme]) -> String {
    let primary = themes.iter()
        .filter(|t| t.occurrences >= 3)
        .take(2)
        .collect::<Vec<_>>();
    
    match primary.len() {
        0 => generate_exploratory_prompt(themes),
        1 => generate_focused_prompt(&primary[0]),
        _ => generate_integration_prompt(&primary[0], &primary[1]),
    }
}

fn generate_integration_prompt(theme1: &CrossEngineTheme, theme2: &CrossEngineTheme) -> String {
    format!(
        "With {} appearing across {} engines and {} appearing across {}, \
         how do these patterns dance together in your experience?",
        theme1.name, theme1.occurrences,
        theme2.name, theme2.occurrences
    )
}
```

---

## API Endpoints

### Execute Workflow

```
POST /api/v1/workflows/{workflow_id}/execute
```

**Request:**
```json
{
  "birth_data": {
    "date": "1990-03-15",
    "time": "14:30",
    "latitude": 40.7128,
    "longitude": -74.0060,
    "timezone": "America/New_York"
  },
  "current_time": "2025-01-15T12:00:00Z",
  "options": {}
}
```

**Response:**
```json
{
  "workflow_id": "birth-blueprint",
  "engine_outputs": {
    "numerology": { "result": {...}, "witness_prompt": "..." },
    "human-design": { "result": {...}, "witness_prompt": "..." },
    "gene-keys": { "result": {...}, "witness_prompt": "..." }
  },
  "synthesis": {
    "primary_themes": [
      {"theme": "Leadership", "occurrences": 3, "sources": [...]}
    ],
    "alignments": [...],
    "tensions": [...],
    "narrative": "Your birth blueprint reveals...",
    "witness_prompt": "With leadership appearing across all three systems..."
  },
  "total_time_ms": 45.2,
  "timestamp": "2025-01-15T12:00:01Z"
}
```

### List Workflows

```
GET /api/v1/workflows
```

**Response:**
```json
{
  "workflows": [
    {
      "id": "birth-blueprint",
      "name": "Birth Blueprint",
      "description": "Core identity mapping through birth data",
      "engines": ["numerology", "human-design", "gene-keys"],
      "ttl_seconds": 86400
    },
    ...
  ]
}
```

### Get Workflow Definition

```
GET /api/v1/workflows/{workflow_id}
```

---

**Last Updated**: 2026-01
**Workflow Version**: 2.0.0 (Wave 2)
