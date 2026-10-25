// Vetted allowlist of non-media engines and workflows exposed to ChatGPT.
//
// Derivation is documented in docs/plans/chatgpt-plugin/CODE-AUDIT.md and
// PLUGIN-SPEC.md: financial-biosensor, raaga, sigil-forge, biofield,
// biofield-capture, face-reading, and any report/witness/assets/media surface
// are DENIED. Unknown IDs are DENIED.
//
// nadabrahman and sacred-geometry are removed: insufficient verification that
// question-only operation (without birth_data or location) is correct. Claims
// of "question-only" are unverified against the actual engine contract.
//
// Source-of-truth for the remaining engine IDs is
// `crates/noesis-orchestrator/src/lib.rs:255-275`. Source-of-truth for the
// canonical workflows is `crates/noesis-orchestrator/src/workflow/registry.rs`.

export const DENIED_ENGINE_REASONS: Record<string, string> = {
  biofield: "biofield capture requires media consent and camera pipeline",
  "biofield-capture": "biofield capture requires media consent and camera pipeline",
  "face-reading": "face reading requires media consent and camera pipeline",
  raaga: "raaga engine involves audio media capture",
  "sigil-forge": "sigil-forge produces generative visual media",
  "financial-biosensor":
    "financial biosensor composes higher-risk downstream advice",
  nadabrahman:
    "nadabrahman removed: insufficient verification of question-only operation",
  "sacred-geometry":
    "sacred-geometry removed: insufficient verification of question-only operation",
};

export const ALLOWED_ENGINE_IDS = [
  "numerology",
  "human-design",
  "gene-keys",
  "vimshottari",
  "panchanga",
  "vedic-clock",
  "biorhythm",
  "transits",
  "enneagram",
  "tarot",
  "i-ching",
] as const;

export type AllowedEngineId = (typeof ALLOWED_ENGINE_IDS)[number];

const ALLOWED_SET = new Set<string>(ALLOWED_ENGINE_IDS);

export function isEngineAllowed(id: string): id is AllowedEngineId {
  return ALLOWED_SET.has(id);
}

export function denyReason(id: string): string {
  if (Object.prototype.hasOwnProperty.call(DENIED_ENGINE_REASONS, id)) {
    return DENIED_ENGINE_REASONS[id];
  }
  return "engine is not in the reviewed non-media allowlist";
}

// Workflow allowlist: only workflows whose engine_ids are all in ALLOWED_SET.
// Verified against crates/noesis-orchestrator/src/workflow/registry.rs.
export const ALLOWED_WORKFLOW_IDS = [
  "decision-support", // tarot, i-ching, human-design, enneagram, gene-keys — all safe
] as const;

export type AllowedWorkflowId = (typeof ALLOWED_WORKFLOW_IDS)[number];

const ALLOWED_WORKFLOW_SET = new Set<string>(ALLOWED_WORKFLOW_IDS);

export function isWorkflowAllowed(id: string): id is AllowedWorkflowId {
  return ALLOWED_WORKFLOW_SET.has(id);
}

// Canonical safe non-media engine set for each allowed workflow. Any workflow
// whose live metadata lists engines outside this map is treated as tainted and
// its `run_workflow` request is refused before POST.
export const ALLOWED_WORKFLOW_ENGINES: Record<
  AllowedWorkflowId,
  readonly AllowedEngineId[]
> = {
  "decision-support": ["tarot", "i-ching", "human-design", "enneagram", "gene-keys"],
};
