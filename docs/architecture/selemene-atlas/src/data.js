// Source snapshot with proposed Pancha Kosha correspondences. See README.md.
export const meta = {
  "title": "Selemene / Living Systems Atlas",
  "snapshot": "12 Sep 2026",
  "revision": "9a05f5cc90777f083d7065e491484ef727aeada1",
  "engineCount": 19,
  "publicMirrors": 17,
  "workflowCount": 6,
  "duration": 24,
  "runtimeLanguages": [
    "Rust",
    "TypeScript",
    "Python"
  ],
  "koshaMapping": "Proposed editorial correspondence — not a runtime contract or physiological classification.",
  "koshaReference": "https://www.swami-krishnananda.org/panch/panch_03.html"
};

export const families = [
  {
    "id": "annamaya",
    "label": "Annamaya",
    "color": "#C5A78A"
  },
  {
    "id": "pranamaya",
    "label": "Pranamaya",
    "color": "#58C4B2"
  },
  {
    "id": "manomaya",
    "label": "Manomaya",
    "color": "#789DE0"
  },
  {
    "id": "vijnanamaya",
    "label": "Vijnanamaya",
    "color": "#B29CD6"
  },
  {
    "id": "anandamaya",
    "label": "Anandamaya",
    "color": "#D4B65F"
  },
  {
    "id": "core",
    "label": "Integration",
    "color": "#C5A017"
  },
  {
    "id": "infra",
    "label": "Infrastructure",
    "color": "#6689B0"
  },
  {
    "id": "connected",
    "label": "Connected systems",
    "color": "#D4B785"
  }
];

export const koshas = [
  {
    "id": "annamaya",
    "label": "Annamaya",
    "meaning": "Physical body",
    "number": "01",
    "color": "#C5A78A",
    "description": "The material and sensory sheath. In this atlas, capture and visible form become the physical entry points.",
    "engines": [
      "face-reading",
      "biofield-capture"
    ]
  },
  {
    "id": "pranamaya",
    "label": "Pranamaya",
    "meaning": "Vital rhythm",
    "number": "02",
    "color": "#58C4B2",
    "description": "The vital sheath. Biofield, cycles and traditional time are grouped as lenses on rhythm and energetic pattern.",
    "engines": [
      "biofield",
      "biorhythm",
      "vedic-clock"
    ]
  },
  {
    "id": "manomaya",
    "label": "Manomaya",
    "meaning": "Mind & symbol",
    "number": "03",
    "color": "#789DE0",
    "description": "The mental sheath. Symbolic mirrors organize impressions, tendencies, images and meaning-making.",
    "engines": [
      "tarot",
      "i-ching",
      "enneagram",
      "sigil-forge"
    ]
  },
  {
    "id": "vijnanamaya",
    "label": "Vijnanamaya",
    "meaning": "Discernment",
    "number": "04",
    "color": "#B29CD6",
    "description": "The sheath of discernment. Structured pattern systems and astronomical timing support comparative reflection.",
    "engines": [
      "human-design",
      "gene-keys",
      "numerology",
      "panchanga",
      "vimshottari",
      "transits",
      "financial-biosensor"
    ]
  },
  {
    "id": "anandamaya",
    "label": "Anandamaya",
    "meaning": "Contemplation",
    "number": "05",
    "color": "#D4B65F",
    "description": "Traditionally the bliss sheath. Sound and sacred form are placed here as contemplative correspondences; no engine measures bliss.",
    "engines": [
      "nadabrahman",
      "raaga",
      "sacred-geometry"
    ]
  }
];

export const nodes = [
  {
    "id": "panchanga",
    "label": "Panchanga",
    "kind": "engine",
    "family": "vijnanamaya",
    "anatomy": "Circadian axis",
    "description": "Five calendar limbs derived from the Sun, Moon and weekday.",
    "source": "crates/engine-panchanga/src/lib.rs",
    "evidence": "Source-defined runtime identity",
    "runtime": "Rust",
    "body": [
      -0.32,
      2.35,
      0.35
    ],
    "field": [
      2.8213692110038697,
      -3.8832815729997483,
      1
    ],
    "kosha": "vijnanamaya",
    "koshaRationale": "Vijnanamaya groups precise reference systems. Panchanga derives five calendar limbs from astronomical inputs."
  },
  {
    "id": "vedic-clock",
    "label": "Vedic Clock",
    "kind": "engine",
    "family": "pranamaya",
    "anatomy": "Circadian axis",
    "description": "A local-hour lens joining the TCM organ clock, dosha cycles and optional calendar qualities.",
    "source": "crates/engine-vedic-clock/src/lib.rs",
    "evidence": "Source-defined runtime identity",
    "runtime": "Rust",
    "body": [
      0.35,
      2.12,
      0.42
    ],
    "field": [
      -3.946224397349012,
      0.6536918278267118,
      1.5
    ],
    "kosha": "pranamaya",
    "koshaRationale": "Pranamaya groups temporal rhythms. Traditional divisions of time become a correspondence with recurring daily cadence."
  },
  {
    "id": "vimshottari",
    "label": "Vimshottari",
    "kind": "engine",
    "family": "vijnanamaya",
    "anatomy": "Temporal spine",
    "description": "A nested planetary-period clock seeded by sidereal natal Moon position.",
    "source": "crates/engine-vimshottari/src/lib.rs",
    "evidence": "Source-defined runtime identity",
    "runtime": "Rust",
    "body": [
      -0.16,
      1.72,
      0.42
    ],
    "field": [
      2.8083321851317353,
      -2.8483803007943327,
      1.25
    ],
    "kosha": "vijnanamaya",
    "koshaRationale": "Vijnanamaya groups structured timing. Vimshottari organizes an interpretive sequence of planetary periods."
  },
  {
    "id": "transits",
    "label": "Transits",
    "kind": "engine",
    "family": "vijnanamaya",
    "anatomy": "Temporal spine",
    "description": "Two planetary reference planes compared through aspects, retrogrades and Saturn-cycle rules.",
    "source": "crates/engine-transits/src/lib.rs",
    "evidence": "Source-defined runtime identity",
    "runtime": "Rust",
    "body": [
      0.23,
      1.47,
      -0.3
    ],
    "field": [
      3.8429451295445753,
      -2.876068971931937,
      1.5
    ],
    "kosha": "vijnanamaya",
    "koshaRationale": "Vijnanamaya connects calculation with comparison. Transits supplies astronomical positions and their interpreted relationships."
  },
  {
    "id": "biorhythm",
    "label": "Biorhythm",
    "kind": "engine",
    "family": "pranamaya",
    "anatomy": "Rhythmic axis",
    "description": "Six date-seeded sine cycles, combined scores and an optional future sampling window.",
    "source": "crates/engine-biorhythm/src/lib.rs",
    "evidence": "Source-defined runtime identity",
    "runtime": "Rust",
    "body": [
      0.33,
      -0.03,
      0.49
    ],
    "field": [
      -4.565071278216736,
      1.483281572999748,
      1.25
    ],
    "kosha": "pranamaya",
    "koshaRationale": "Pranamaya groups lenses on changing rhythms. Biorhythm uses recurring cycles as a reflective model, not a physiological sensor."
  },
  {
    "id": "numerology",
    "label": "Numerology",
    "kind": "engine",
    "family": "vijnanamaya",
    "anatomy": "Pattern recognition",
    "description": "Name and date arithmetic exposed as traceable reduction chains.",
    "source": "crates/engine-numerology/src/lib.rs",
    "evidence": "Source-defined runtime identity",
    "runtime": "Rust",
    "body": [
      0.31,
      2.75,
      0.42
    ],
    "field": [
      1.841148274901361,
      -3.5510805439792184,
      1.5
    ],
    "kosha": "vijnanamaya",
    "koshaRationale": "Vijnanamaya groups structured interpretation. Numerology maps inputs into a numerical pattern vocabulary."
  },
  {
    "id": "human-design",
    "label": "Human Design",
    "kind": "engine",
    "family": "vijnanamaya",
    "anatomy": "Bodygraph axis",
    "description": "A BodyGraph assembled from two sets of thirteen planetary activations.",
    "source": "crates/engine-human-design/src/lib.rs",
    "evidence": "Source-defined runtime identity",
    "runtime": "Rust",
    "body": [
      -0.42,
      0.1,
      0.51
    ],
    "field": [
      0.7095000726761322,
      -3.9365733381803727,
      1
    ],
    "kosha": "vijnanamaya",
    "koshaRationale": "Vijnanamaya is the lens of discernment. Human Design is placed here for its structured synthesis of multiple pattern systems."
  },
  {
    "id": "gene-keys",
    "label": "Gene Keys",
    "kind": "engine",
    "family": "vijnanamaya",
    "anatomy": "Reflective axis",
    "description": "Four Human Design gates expanded into an Activation Sequence and contemplative vocabulary.",
    "source": "crates/engine-gene-keys/src/lib.rs",
    "evidence": "Source-defined runtime identity",
    "runtime": "Rust",
    "body": [
      0.36,
      0.64,
      0.51
    ],
    "field": [
      1.5477687835904619,
      -4.543612196539544,
      1.25
    ],
    "kosha": "vijnanamaya",
    "koshaRationale": "Vijnanamaya supports reflective discrimination. Gene Keys interprets a structured set of themes for contemplation."
  },
  {
    "id": "enneagram",
    "label": "Enneagram",
    "kind": "engine",
    "family": "manomaya",
    "anatomy": "Reflective mind",
    "description": "A questionnaire or type lookup joined to nine-pattern taxonomy and reflective movement.",
    "source": "ts-engines/src/engines/enneagram/index.ts",
    "evidence": "Source-defined runtime identity",
    "runtime": "TypeScript / Bun",
    "body": [
      -0.32,
      2.78,
      0.42
    ],
    "field": [
      -2.1020538978805132,
      -3.4031411093878168,
      1.5
    ],
    "kosha": "manomaya",
    "koshaRationale": "Manomaya groups lenses on mental and emotional tendencies. Enneagram patterns are placed here as reflective descriptions."
  },
  {
    "id": "tarot",
    "label": "Tarot",
    "kind": "engine",
    "family": "manomaya",
    "anatomy": "Left hand / symbolic action",
    "description": "A seeded deck draw assembled into positions, orientations and local symbolic readings.",
    "source": "ts-engines/src/engines/tarot/index.ts",
    "evidence": "Source-defined runtime identity",
    "runtime": "TypeScript / Bun",
    "body": [
      -2.65,
      1.72,
      0.28
    ],
    "field": [
      -3.0138657546979535,
      -2.6299454771266526,
      1
    ],
    "kosha": "manomaya",
    "koshaRationale": "Manomaya concerns impressions and meaning. Tarot works through symbolic images and their interpretation."
  },
  {
    "id": "i-ching",
    "label": "I Ching",
    "kind": "engine",
    "family": "manomaya",
    "anatomy": "Right hand / symbolic action",
    "description": "A seeded consultation scaffold around a 64-hexagram text library.",
    "source": "ts-engines/src/engines/i-ching/index.ts",
    "evidence": "Source-defined runtime identity",
    "runtime": "TypeScript / Bun",
    "body": [
      2.65,
      1.72,
      0.28
    ],
    "field": [
      -3.104410980501628,
      -3.660960593087683,
      1.25
    ],
    "kosha": "manomaya",
    "koshaRationale": "Manomaya concerns patterns of thought and interpretation. Hexagrams provide a symbolic vocabulary for reflecting on change."
  },
  {
    "id": "sacred-geometry",
    "label": "Sacred Geometry",
    "kind": "engine",
    "family": "anandamaya",
    "anatomy": "Structural axis",
    "description": "A form-library browser with symbolism, meditation and an optional SVG-status check.",
    "source": "ts-engines/src/engines/sacred-geometry/index.ts",
    "evidence": "Source-defined runtime identity",
    "runtime": "TypeScript / Bun",
    "body": [
      -0.25,
      -0.8,
      0.45
    ],
    "field": [
      3.576793017013037,
      1.7906847052026709,
      1.5
    ],
    "kosha": "anandamaya",
    "koshaRationale": "Anandamaya is interpreted here through contemplative form. Geometry evokes order and wholeness without measuring a state of consciousness."
  },
  {
    "id": "sigil-forge",
    "label": "Sigil Forge",
    "kind": "engine",
    "family": "manomaya",
    "anatomy": "Expressive hand",
    "description": "Intention reduction and ritual guidance with an optional external image-generation branch.",
    "source": "ts-engines/src/engines/sigil-forge/index.ts",
    "evidence": "Source-defined runtime identity",
    "runtime": "TypeScript / Bun",
    "body": [
      2.78,
      1.54,
      0.28
    ],
    "field": [
      -1.883869255867114,
      -4.4148654143471555,
      1
    ],
    "kosha": "manomaya",
    "koshaRationale": "Manomaya connects intention with symbolic form. Sigil Forge transforms an idea into an expressive image."
  },
  {
    "id": "raaga",
    "label": "Raaga",
    "kind": "engine",
    "family": "anandamaya",
    "anatomy": "Voice / expression",
    "description": "A 72-Melakarta musical catalog expanded into swaras, ratios and optional audio.",
    "source": "ts-engines/src/engines/raaga/index.ts",
    "evidence": "Source-defined runtime identity",
    "runtime": "TypeScript / Bun",
    "body": [
      0.28,
      1.99,
      0.38
    ],
    "field": [
      4.565071278216737,
      1.4832815729997464,
      1.25
    ],
    "kosha": "anandamaya",
    "koshaRationale": "Anandamaya is interpreted here through contemplation. Musical form and sound provide the correspondence, not evidence of a bliss state."
  },
  {
    "id": "nadabrahman",
    "label": "NadaBrahman",
    "kind": "engine",
    "family": "anandamaya",
    "anatomy": "Resonant thorax",
    "description": "A recommendation lattice selecting musical material by time, dosha and rasa.",
    "source": "crates/engine-nadabrahman/src/lib.rs",
    "evidence": "Source-defined runtime identity",
    "runtime": "Rust",
    "body": [
      -0.48,
      1.21,
      0.5
    ],
    "field": [
      3.9462243973490123,
      0.6536918278267085,
      1
    ],
    "kosha": "anandamaya",
    "koshaRationale": "Anandamaya traditionally names the bliss sheath. Sound is placed here as a contemplative correspondence; this engine does not measure bliss."
  },
  {
    "id": "face-reading",
    "label": "Face Reading",
    "kind": "engine",
    "family": "annamaya",
    "anatomy": "Perceptual face",
    "description": "A consent-gated landmark adapter plus deterministic fallback and traditional symbolic interpretation.",
    "source": "crates/engine-face-reading/src/lib.rs",
    "evidence": "Source-defined runtime identity",
    "runtime": "Rust",
    "body": [
      0,
      2.6,
      0.65
    ],
    "field": [
      0.2997188290909694,
      3.98875527244883,
      1
    ],
    "kosha": "annamaya",
    "koshaRationale": "Annamaya concerns embodied form. This engine reads visible facial features, making physical appearance its entry point."
  },
  {
    "id": "biofield",
    "label": "Biofield",
    "kind": "engine",
    "family": "pranamaya",
    "anatomy": "Somatic field",
    "description": "A symbolic field computed from planetary correspondences, with an explicitly separate mock branch.",
    "source": "crates/engine-biofield/src/lib.rs",
    "evidence": "Source-defined runtime identity",
    "runtime": "Rust",
    "body": [
      0.75,
      0.52,
      0.28
    ],
    "field": [
      -3.576793017013037,
      1.7906847052026706,
      1
    ],
    "kosha": "pranamaya",
    "koshaRationale": "Pranamaya is used here as a symbolic lens on energetic pattern. The software identity is distinct from a measured vital force."
  },
  {
    "id": "biofield-capture",
    "label": "Biofield Capture",
    "kind": "engine",
    "family": "annamaya",
    "anatomy": "Capture boundary",
    "description": "An owner-scoped resolver for a previously persisted capture analysis.",
    "source": "crates/engine-biofield-capture/src/lib.rs",
    "evidence": "Source-defined runtime identity",
    "runtime": "Rust",
    "body": [
      -0.72,
      -0.27,
      0.33
    ],
    "field": [
      -0.35966259490916275,
      4.7865063269385955,
      1.25
    ],
    "kosha": "annamaya",
    "koshaRationale": "Annamaya is the material entry layer. This engine persists captured observations and their metadata before interpretation."
  },
  {
    "id": "financial-biosensor",
    "label": "Financial Biosensor",
    "kind": "engine",
    "family": "vijnanamaya",
    "anatomy": "Decision axis",
    "description": "A declared decision-reflection composite over five engines and one optional biometric sample.",
    "source": "crates/engine-financial-biosensor/src/lib.rs",
    "evidence": "Source-defined runtime identity",
    "runtime": "Rust",
    "body": [
      0.28,
      -0.64,
      0.43
    ],
    "field": [
      3.5246561451830227,
      -1.8912427285315743,
      1
    ],
    "kosha": "vijnanamaya",
    "koshaRationale": "Vijnanamaya is used for comparative discernment. Financial Biosensor is an interpretive signal lens; its placement makes no claim of measured bodily wisdom."
  },
  {
    "id": "selemene",
    "label": "Selemene Engine",
    "kind": "core",
    "family": "core",
    "anatomy": "Heart / integration",
    "description": "WorkflowOrchestrator centralizes engine registration, phase gates and workflow execution. This is the integration heart of the atlas.",
    "source": "crates/noesis-orchestrator/src/lib.rs",
    "evidence": "Source-defined component",
    "runtime": "Rust",
    "body": [
      0.04,
      1.05,
      0.65
    ],
    "field": [
      0,
      1.05,
      0.8
    ]
  },
  {
    "id": "api",
    "label": "Noesis API",
    "kind": "core",
    "family": "core",
    "anatomy": "Spine / communication",
    "description": "Axum HTTP boundary for engines, workflows and authenticated consumers.",
    "source": "crates/noesis-api/src/lib.rs",
    "evidence": "Source-defined component",
    "runtime": "Rust / Axum",
    "body": [
      0,
      0.65,
      -0.3
    ],
    "field": [
      0,
      3.5,
      -0.65
    ]
  },
  {
    "id": "ts-service",
    "label": "TypeScript Engines",
    "kind": "core",
    "family": "core",
    "anatomy": "Symbolic hemisphere",
    "description": "Bun service registers Tarot, I Ching, Enneagram, Sacred Geometry, Sigil Forge and Raaga.",
    "source": "ts-engines/src/index.ts",
    "evidence": "Source-defined component",
    "runtime": "TypeScript / Bun",
    "body": [
      0.6,
      1.62,
      0.1
    ],
    "field": [
      2.25,
      2.3,
      -1.2
    ]
  },
  {
    "id": "mediapipe",
    "label": "MediaPipe Sidecar",
    "kind": "core",
    "family": "core",
    "anatomy": "Eyes / perception",
    "description": "Python face-mesh service declared in Docker Compose. Availability is not measured by this atlas.",
    "source": "python-services/Dockerfile.mediapipe",
    "evidence": "Source-defined component",
    "runtime": "Python",
    "body": [
      -0.45,
      2.53,
      -0.1
    ],
    "field": [
      -2.2,
      3.4,
      -1.2
    ]
  },
  {
    "id": "biofield-cv",
    "label": "Biofield CV Sidecar",
    "kind": "core",
    "family": "core",
    "anatomy": "Surrounding field",
    "description": "Python image-analysis service, separate from the native Biofield calculation and stored-capture identities.",
    "source": "python-services/Dockerfile.biofield",
    "evidence": "Source-defined component",
    "runtime": "Python",
    "body": [
      -1.5,
      0.43,
      -0.5
    ],
    "field": [
      -2.5,
      -1.5,
      -1.2
    ]
  },
  {
    "id": "witness",
    "label": "Witness Pipeline",
    "kind": "core",
    "family": "core",
    "anatomy": "Witness / integration",
    "description": "Narrative reflection is composed over engine results. Its interpretation does not replace deterministic output.",
    "source": "packages/witness-pipeline/README.md",
    "evidence": "Source-defined component",
    "runtime": "TypeScript",
    "body": [
      0,
      3.5,
      -0.3
    ],
    "field": [
      0,
      4.35,
      0
    ]
  },
  {
    "id": "sdk",
    "label": "Engine SDK",
    "kind": "core",
    "family": "core",
    "anatomy": "Nervous system interface",
    "description": "Typed client contract for consumers of engine results, capabilities, consent and provenance.",
    "source": "packages/noesis-engine-sdk/README.md",
    "evidence": "Source-defined component",
    "runtime": "TypeScript",
    "body": [
      1.65,
      1.67,
      -0.25
    ],
    "field": [
      2.5,
      0.5,
      -1.5
    ]
  },
  {
    "id": "railway",
    "label": "Railway",
    "kind": "infra",
    "family": "infra",
    "anatomy": "Outer support field",
    "description": "Deployment configuration describes the service hosting plane. A deployment guide is not a current health receipt.",
    "source": "docs/deployment/RAILWAY.md",
    "evidence": "Declared configuration",
    "runtime": "Hosting configuration",
    "body": [
      -3.1,
      2.42,
      -0.2
    ],
    "field": [
      -6.1,
      3.5,
      -1
    ]
  },
  {
    "id": "gateway",
    "label": "Cloudflare Gateway",
    "kind": "infra",
    "family": "infra",
    "anatomy": "Protective boundary",
    "description": "Worker gateway uses a KV secret binding and mediates the API access boundary.",
    "source": "workers/selemene-gw/src/index.js",
    "evidence": "Declared configuration",
    "runtime": "Cloudflare Worker",
    "body": [
      -3.5,
      0.94,
      -0.2
    ],
    "field": [
      -6.1,
      1.8,
      -1
    ]
  },
  {
    "id": "postgres",
    "label": "PostgreSQL",
    "kind": "infra",
    "family": "infra",
    "anatomy": "Long-term memory",
    "description": "Durable relational storage declared for the API service in Docker Compose.",
    "source": "docker-compose.yml",
    "evidence": "Declared configuration",
    "runtime": "PostgreSQL 16",
    "body": [
      -3.22,
      -0.48,
      -0.2
    ],
    "field": [
      -6.1,
      0.1,
      -1
    ]
  },
  {
    "id": "redis",
    "label": "Redis",
    "kind": "infra",
    "family": "infra",
    "anatomy": "Working memory",
    "description": "L2 cache within the engine-agnostic L1 memory / L2 Redis / L3 disk cache.",
    "source": "crates/noesis-cache/src/lib.rs",
    "evidence": "Source-defined component",
    "runtime": "Redis / Rust",
    "body": [
      -2.7,
      -1.78,
      -0.2
    ],
    "field": [
      -6.1,
      -1.6,
      -1
    ]
  },
  {
    "id": "ephemeris",
    "label": "Swiss Ephemeris",
    "kind": "infra",
    "family": "infra",
    "anatomy": "Astronomical reference field",
    "description": "Bundled ephemeris data supports the astronomical calculations.",
    "source": "crates/noesis-cache/src/ephemeris.rs",
    "evidence": "Source-defined component",
    "runtime": "Local data",
    "body": [
      0,
      3.97,
      -0.65
    ],
    "field": [
      0,
      5.3,
      -1
    ]
  },
  {
    "id": "llm-proxy",
    "label": "LLM Proxy",
    "kind": "infra",
    "family": "infra",
    "anatomy": "Narrative field",
    "description": "Cloudflare Worker mediates configured language-model requests; no provider request is made by this atlas.",
    "source": "workers/llm-proxy/wrangler.toml",
    "evidence": "Declared configuration",
    "runtime": "Cloudflare Worker",
    "body": [
      3.3,
      2.6,
      -0.2
    ],
    "field": [
      6.1,
      3.5,
      -1
    ]
  },
  {
    "id": "pattern-memory",
    "label": "Pattern Memory",
    "kind": "infra",
    "family": "infra",
    "anatomy": "Reflective memory field",
    "description": "Worker scaffold declares Vectorize, R2, D1, KV and Workers AI. Placeholder bindings do not establish deployment.",
    "source": "workers/pattern-memory/wrangler.toml",
    "evidence": "Declared configuration",
    "runtime": "Cloudflare / declared scaffold",
    "body": [
      3.5,
      0.8,
      -0.2
    ],
    "field": [
      6.1,
      1.8,
      -1
    ]
  },
  {
    "id": "metrics",
    "label": "Metrics & Tracing",
    "kind": "infra",
    "family": "infra",
    "anatomy": "Feedback field",
    "description": "Metrics crate and optional telemetry provide an observation boundary in the source.",
    "source": "crates/noesis-metrics/src/lib.rs",
    "evidence": "Source-defined component",
    "runtime": "Rust / OpenTelemetry",
    "body": [
      3.1,
      -1.03,
      -0.2
    ],
    "field": [
      6.1,
      0.1,
      -1
    ]
  },
  {
    "id": "auth",
    "label": "Authentication",
    "kind": "infra",
    "family": "infra",
    "anatomy": "Membrane / access",
    "description": "Authentication crate provides the API access boundary.",
    "source": "crates/noesis-auth/src/lib.rs",
    "evidence": "Source-defined component",
    "runtime": "Rust",
    "body": [
      -2.16,
      0.02,
      -0.75
    ],
    "field": [
      -4.95,
      -3.1,
      -0.4
    ]
  },
  {
    "id": "contracts",
    "label": "Contract Authority",
    "kind": "infra",
    "family": "infra",
    "anatomy": "Connective tissue",
    "description": "Language-neutral v1 schemas define request, result, capability, consent, error and provenance shapes.",
    "source": "contracts/v1/manifest.json",
    "evidence": "Source-defined component",
    "runtime": "JSON Schema",
    "body": [
      2.45,
      -2.4,
      -0.3
    ],
    "field": [
      4.95,
      -3.1,
      -0.4
    ]
  },
  {
    "id": "sankalpa",
    "label": "Sankalpa",
    "kind": "connected",
    "family": "connected",
    "anatomy": "Embodied practice",
    "description": "Separate consuming application referenced as the home of the Biofield experience. This atlas does not connect to the app.",
    "source": "docs/design/biofield-web/README.md",
    "evidence": "Documented consumer boundary",
    "runtime": "External consumer",
    "body": [
      -4.35,
      -2.65,
      -1
    ],
    "field": [
      -6.6,
      -4.4,
      0
    ]
  },
  {
    "id": "admin",
    "label": "Admin Web",
    "kind": "connected",
    "family": "connected",
    "anatomy": "Operator awareness",
    "description": "Administrative web surface in apps/admin-web.",
    "source": "apps/admin-web/README.md",
    "evidence": "Documented consumer boundary",
    "runtime": "Next.js",
    "body": [
      -2.1,
      -3.5,
      -1
    ],
    "field": [
      -3.9,
      -4.4,
      0
    ]
  },
  {
    "id": "tui",
    "label": "Noesis TUI",
    "kind": "connected",
    "family": "connected",
    "anatomy": "Terminal interface",
    "description": "Terminal client crate for the Noesis ecosystem.",
    "source": "crates/noesis-tui/Cargo.toml",
    "evidence": "Documented consumer boundary",
    "runtime": "Rust",
    "body": [
      0,
      -3.98,
      -1
    ],
    "field": [
      -1.3,
      -4.4,
      0
    ]
  },
  {
    "id": "agent-bridges",
    "label": "Agent Bridges",
    "kind": "connected",
    "family": "connected",
    "anatomy": "Tool-making hands",
    "description": "CLI generates Claude, OpenAI and LangChain-compatible tools from the API schema.",
    "source": "bridges/README.md",
    "evidence": "Documented consumer boundary",
    "runtime": "TypeScript / Python",
    "body": [
      2,
      -3.5,
      -1
    ],
    "field": [
      1.3,
      -4.4,
      0
    ]
  },
  {
    "id": "hermes",
    "label": "Hermes",
    "kind": "connected",
    "family": "connected",
    "anatomy": "Agent interface",
    "description": "NousResearch Hermes function-calling integration.",
    "source": "bridges/hermes/README.md",
    "evidence": "Documented consumer boundary",
    "runtime": "Python",
    "body": [
      4.3,
      -2.65,
      -1
    ],
    "field": [
      3.9,
      -4.4,
      0
    ]
  },
  {
    "id": "langchain",
    "label": "LangChain / CrewAI",
    "kind": "connected",
    "family": "connected",
    "anatomy": "Agent interface",
    "description": "StructuredTool bridge generated from Noesis OpenAPI.",
    "source": "bridges/langchain/README.md",
    "evidence": "Documented consumer boundary",
    "runtime": "Python",
    "body": [
      4.6,
      -0.13,
      -1
    ],
    "field": [
      6.6,
      -4.4,
      0
    ]
  }
];

export const edges = [
  {
    "from": "selemene",
    "to": "panchanga",
    "label": "Engine registration",
    "source": "crates/noesis-orchestrator/src/lib.rs"
  },
  {
    "from": "selemene",
    "to": "vedic-clock",
    "label": "Engine registration",
    "source": "crates/noesis-orchestrator/src/lib.rs"
  },
  {
    "from": "selemene",
    "to": "vimshottari",
    "label": "Engine registration",
    "source": "crates/noesis-orchestrator/src/lib.rs"
  },
  {
    "from": "selemene",
    "to": "transits",
    "label": "Engine registration",
    "source": "crates/noesis-orchestrator/src/lib.rs"
  },
  {
    "from": "selemene",
    "to": "biorhythm",
    "label": "Engine registration",
    "source": "crates/noesis-orchestrator/src/lib.rs"
  },
  {
    "from": "selemene",
    "to": "numerology",
    "label": "Engine registration",
    "source": "crates/noesis-orchestrator/src/lib.rs"
  },
  {
    "from": "selemene",
    "to": "human-design",
    "label": "Engine registration",
    "source": "crates/noesis-orchestrator/src/lib.rs"
  },
  {
    "from": "selemene",
    "to": "gene-keys",
    "label": "Engine registration",
    "source": "crates/noesis-orchestrator/src/lib.rs"
  },
  {
    "from": "ts-service",
    "to": "enneagram",
    "label": "Engine registration",
    "source": "ts-engines/src/index.ts"
  },
  {
    "from": "ts-service",
    "to": "tarot",
    "label": "Engine registration",
    "source": "ts-engines/src/index.ts"
  },
  {
    "from": "ts-service",
    "to": "i-ching",
    "label": "Engine registration",
    "source": "ts-engines/src/index.ts"
  },
  {
    "from": "ts-service",
    "to": "sacred-geometry",
    "label": "Engine registration",
    "source": "ts-engines/src/index.ts"
  },
  {
    "from": "ts-service",
    "to": "sigil-forge",
    "label": "Engine registration",
    "source": "ts-engines/src/index.ts"
  },
  {
    "from": "ts-service",
    "to": "raaga",
    "label": "Engine registration",
    "source": "ts-engines/src/index.ts"
  },
  {
    "from": "selemene",
    "to": "nadabrahman",
    "label": "Engine registration",
    "source": "crates/noesis-orchestrator/src/lib.rs"
  },
  {
    "from": "selemene",
    "to": "face-reading",
    "label": "Engine registration",
    "source": "crates/noesis-orchestrator/src/lib.rs"
  },
  {
    "from": "selemene",
    "to": "biofield",
    "label": "Engine registration",
    "source": "crates/noesis-orchestrator/src/lib.rs"
  },
  {
    "from": "selemene",
    "to": "biofield-capture",
    "label": "Database-backed capability boundary",
    "source": "crates/noesis-orchestrator/src/lib.rs"
  },
  {
    "from": "selemene",
    "to": "financial-biosensor",
    "label": "Engine registration",
    "source": "crates/noesis-orchestrator/src/lib.rs"
  },
  {
    "from": "api",
    "to": "selemene",
    "label": "Orchestrated calculation",
    "source": "crates/noesis-api/src/lib.rs"
  },
  {
    "from": "selemene",
    "to": "ts-service",
    "label": "TypeScript bridge",
    "source": "crates/noesis-bridge/src/lib.rs"
  },
  {
    "from": "face-reading",
    "to": "mediapipe",
    "label": "Image-analysis dependency",
    "source": "crates/engine-face-reading/src/lib.rs"
  },
  {
    "from": "api",
    "to": "biofield-cv",
    "label": "Live Biofield analysis boundary",
    "source": "crates/noesis-api/src/handlers/biofield.rs"
  },
  {
    "from": "biofield-capture",
    "to": "postgres",
    "label": "Stored readings",
    "source": "crates/noesis-api/src/handlers/biofield.rs"
  },
  {
    "from": "sdk",
    "to": "api",
    "label": "Typed API requests",
    "source": "packages/noesis-engine-sdk/README.md"
  },
  {
    "from": "witness",
    "to": "api",
    "label": "Engine result consumption",
    "source": "packages/witness-pipeline/README.md"
  },
  {
    "from": "witness",
    "to": "llm-proxy",
    "label": "Narrative provider boundary",
    "source": "packages/witness-pipeline/README.md"
  },
  {
    "from": "gateway",
    "to": "api",
    "label": "Gateway requests",
    "source": "workers/selemene-gw/src/index.js"
  },
  {
    "from": "railway",
    "to": "api",
    "label": "Declared hosting",
    "source": "docs/deployment/RAILWAY.md"
  },
  {
    "from": "railway",
    "to": "ts-service",
    "label": "Declared hosting",
    "source": "docs/deployment/RAILWAY.md"
  },
  {
    "from": "api",
    "to": "postgres",
    "label": "Relational storage",
    "source": "docker-compose.yml"
  },
  {
    "from": "api",
    "to": "redis",
    "label": "L2 cache",
    "source": "docker-compose.yml"
  },
  {
    "from": "api",
    "to": "auth",
    "label": "Authenticated boundary",
    "source": "crates/noesis-api/src/lib.rs"
  },
  {
    "from": "api",
    "to": "metrics",
    "label": "Instrumentation",
    "source": "crates/noesis-api/src/lib.rs"
  },
  {
    "from": "sdk",
    "to": "contracts",
    "label": "Schema compatibility",
    "source": "packages/noesis-engine-sdk/src/contract-v1.ts"
  },
  {
    "from": "panchanga",
    "to": "ephemeris",
    "label": "Astronomical reference",
    "source": "crates/engine-panchanga/src/lib.rs"
  },
  {
    "from": "transits",
    "to": "ephemeris",
    "label": "Astronomical reference",
    "source": "crates/engine-transits/src/lib.rs"
  },
  {
    "from": "gene-keys",
    "to": "human-design",
    "label": "Constructor dependency",
    "source": "crates/noesis-orchestrator/src/lib.rs"
  },
  {
    "from": "vimshottari",
    "to": "human-design",
    "label": "Retained constructor reference; not used by calculation",
    "source": "crates/noesis-orchestrator/src/lib.rs"
  },
  {
    "from": "sankalpa",
    "to": "sdk",
    "label": "Documented consumer integration",
    "source": "packages/noesis-engine-sdk/README.md"
  },
  {
    "from": "admin",
    "to": "api",
    "label": "Admin API boundary",
    "source": "apps/admin-web/README.md"
  },
  {
    "from": "tui",
    "to": "api",
    "label": "Client boundary",
    "source": "crates/noesis-tui/Cargo.toml"
  },
  {
    "from": "agent-bridges",
    "to": "api",
    "label": "OpenAPI tool generation",
    "source": "bridges/README.md"
  },
  {
    "from": "hermes",
    "to": "api",
    "label": "Function-calling tools",
    "source": "bridges/hermes/README.md"
  },
  {
    "from": "langchain",
    "to": "api",
    "label": "StructuredTool boundary",
    "source": "bridges/langchain/README.md"
  },
  {
    "from": "financial-biosensor",
    "to": "human-design",
    "label": "Composite source engine",
    "source": "crates/noesis-orchestrator/src/lib.rs"
  },
  {
    "from": "financial-biosensor",
    "to": "gene-keys",
    "label": "Composite source engine",
    "source": "crates/noesis-orchestrator/src/lib.rs"
  },
  {
    "from": "financial-biosensor",
    "to": "vimshottari",
    "label": "Composite source engine",
    "source": "crates/noesis-orchestrator/src/lib.rs"
  },
  {
    "from": "financial-biosensor",
    "to": "transits",
    "label": "Composite source engine",
    "source": "crates/noesis-orchestrator/src/lib.rs"
  },
  {
    "from": "financial-biosensor",
    "to": "biorhythm",
    "label": "Composite source engine",
    "source": "crates/noesis-orchestrator/src/lib.rs"
  }
];
