// ─── Premium Asset Generation (additive SDK surface) ───────────────────────
// Wires to the new /api/v1/assets/generate endpoint and optionally to
// the local @noesis/witness-pipeline for full multi-pass orchestration.
//
// Server call is ALWAYS made (source of manifest/seeds for the result envelope).
// When llm + mode provided, we additionally fetch real engine seeds via the
// client's calculate() and run the real IntegratedReadingOrchestrator.

import type { BirthData, EngineOutput } from './index.js';
import type {
  IntegratedReadingOrchestrator,
  OrchestratorInput,
  OrchestratorOutput,
  ParsedModeDoc,
  SelemeneEngineOutput,
} from '@noesis/witness-pipeline';
import { parseModeDocument } from '@noesis/witness-pipeline';

// Re-export pipeline types so callers can use them directly from the SDK surface (additive).
export type { ParsedModeDoc, OrchestratorOutput, OrchestratorInput, SelemeneEngineOutput } from '@noesis/witness-pipeline';

export type LlmCall = (system: string, user: string, opts: { max_tokens: number }) => Promise<string>;

export interface PremiumAssetInput {
  birth_data?: BirthData;
  mode: string;
  consciousness_level?: number;
  options?: Record<string, unknown>;
}

export interface PremiumAssetPass {
  id: string;
  title: string;
  output: string;
}

export interface PremiumAssetResult {
  mode: string;
  register: string;
  passes: PremiumAssetPass[];
  assembled: string;
  engines_used: string[];
  source_pack?: Record<string, unknown>;
  // When full orchestration was run locally via witness-pipeline
  orchestrator_output?: OrchestratorOutput;
}

// Canonical integrated-reading mode document (for clean "pass mode name" usage).
// Other common modes can be added here or callers can pass a pre-parsed ParsedModeDoc.
const INTEGRATED_READING_DOC = `---
mode: integrated-reading
subject_count:
  min: 1
  max: 2
roles:
  - subject
target_words:
  min: 4200
  max: 5800
architecture: linear
pass_plan:
  - id: structural
    title: Structural Field
    target_words: 1800
    template: structural-pass
  - id: somatic
    title: Somatic Vitality
    target_words: 1200
    template: somatic-pass
  - id: synthesis
    title: Synthesis & Invitation
    target_words: 1500
    template: synthesis-pass
engine_overlay_weights:
  panchanga: 1.0
  vimshottari: 0.9
  human-design: 0.85
  gene-keys: 0.75
  numerology: 0.6
house_overlay: [1, 4, 7, 10]
bridge_mandates:
  - "Braid deterministic facts from multiple engines before any interpretation"
  - "Hold the tension between structure (Aletheios) and flow (Pichet) without forcing resolution"
  - "Surface one concrete, non-prescriptive invitation per major pass"
  - "Name shadow/gift dynamics only when consciousness register supports it"
svg_topology: dyad-arc
register_variants:
  l1_l3:
    target_words:
      min: 3800
      max: 5200
    overrides:
      - pass_id: synthesis
        template: synthesis-pass-l1-l3
  l4_l5:
    target_words:
      min: 4800
      max: 6500
---

## structural-pass

Compose the structural field for {{subject_names}}.

You are writing in register {{register}}.

Context from prior pass (if any):
{{prior_pass}}

Engine overlay weights and focus houses:
{{overlay_summary}}

Bridge mandates to honor:
{{bridge_mandates}}

Recent autoresearch lessons (use only to avoid repeating past dead-ends):
{{lessons_summary}}

Target length: ~{{target_words}} words (pass id: {{pass_id}}).

Write a precise, non-prescriptive mapping of the dominant structural patterns across the engines. Weave panchanga + vimshottari timing with Human Design gates and Gene Key activations. Identify 2-3 "load-bearing" facts that multiple engines converge on. Do not moralize. Leave tension visible.

## somatic-pass

Now layer the somatic and vitality currents for {{subject_names}}.

Register: {{register}}.

Prior structural field (use for braid, do not repeat verbatim):
{{prior_pass}}

Overlay and houses:
{{overlay_summary}}

Mandates:
{{bridge_mandates}}

Lessons:
{{lessons_summary}}

Target ~{{target_words}} words. Pass: {{pass_id}}.

Describe the felt-sense, biorhythmic, and transiting vitality layer. Note where the structural field is supported or resisted somatically. Name one place where the body is being asked to metabolize a timing or gate activation. Stay close to observable data. Avoid therapy-speak.

## synthesis-pass

Synthesize the reading for {{subject_names}} into a coherent mirror.

Register: {{register}}.

Prior passes (structural then somatic):
{{prior_pass}}

Overlay summary:
{{overlay_summary}}

Bridge mandates (especially the non-resolution and invitation rules):
{{bridge_mandates}}

Lessons:
{{lessons_summary}}

Target ~{{target_words}} words. Pass id: {{pass_id}}.

Hold Aletheios (structure) and Pichet (vitality) in the same frame. Surface the primary tension or coherence. Offer 1-2 precise invitations that honor the subject's autonomy. Do not promise outcomes. End on an open question the subject can live into.

## synthesis-pass-l1-l3

Synthesize for {{subject_names}} at the l1_l3 register (earlier consciousness band).

Prior material:
{{prior_pass}}

Focus on clear pattern recognition. Use simpler language. Emphasize observable facts over deep shadow work. One small invitation only. Target ~{{target_words}} words.

## overlay-rules

When describing engine contributions, always cite the specific engine and the concrete data point (e.g., "Panchanga tithi X on date Y" or "HD gate 34 in the sacral center").

Never use the engines to diagnose, predict, or prescribe. The output is a mirror, not a verdict.

When register is l1_l3, avoid or soften Gene Keys shadow language and siddhi language. When l4_l5, you may surface shadow/gift/siddhi dynamics but still as possibilities, not identities.

## lessons

### 2026-06-12 — Over-braiding caused loss of subject voice

**Question:** In multi-engine passes, how do we prevent the synthesis from sounding like a committee report?
**Variants:** Heavy cross-citation / Lighter voice with selective braid / Let engines speak in turn then one final human-voiced pass
**Winner:** Lighter voice with selective braid
**Adopted:** Let each pass have a primary voice; only braid the load-bearing facts that genuinely converge. The final invitation must sound like one coherent person wrote it.
**Reference:** witness-agents v0.9.4 post-mortem

### 2026-06-20 — l1_l3 users disoriented by siddhi language

**Question:** Should register variants affect only length or also depth of shadow/siddhi language?
**Adopted:** Yes — l1_l3 synthesis template must explicitly instruct lighter language and forbid siddhi framing.
`;

const BUILTIN_MODES: Record<string, string> = {
  'integrated-reading': INTEGRATED_READING_DOC,
  'integrated_reading': INTEGRATED_READING_DOC,
};

/** Resolve a mode name (for common modes) or return the provided doc. Purely additive. */
export function resolveModeDoc(mode?: string | ParsedModeDoc): ParsedModeDoc {
  if (mode && typeof mode !== 'string') return mode;
  const key = (mode || 'integrated-reading').toLowerCase().replace(/[^a-z0-9-]/g, '-');
  const content = BUILTIN_MODES[key] ?? BUILTIN_MODES['integrated-reading'];
  if (!content) {
    throw new Error(`Unknown mode '${mode}'. Pass a ParsedModeDoc or use a supported built-in mode name.`);
  }
  return parseModeDocument(content, `${key}.md`);
}

/** Map SDK EngineOutput (from calculate) to the SelemeneEngineOutput shape the pipeline expects. */
export function engineOutputToSelemene(eo: EngineOutput, level: number): SelemeneEngineOutput {
  const wp = eo.witness_prompt ?? eo.witness_prompts?.[0] ?? '';
  const meta = (eo.metadata as Record<string, unknown>) || {};
  return {
    engine_id: eo.engine_id as SelemeneEngineOutput['engine_id'],
    result: eo.result,
    witness_prompt: wp,
    consciousness_level: eo.consciousness_level ?? level,
    metadata: {
      calculation_time_ms: Number((meta as any).calculation_time_ms) || 0,
      backend: String((meta as any).backend || 'selemene'),
      precision_achieved: String((meta as any).precision_achieved || 'standard'),
      cached: Boolean((meta as any).cached),
      timestamp: String((meta as any).timestamp || new Date().toISOString()),
      engine_version: String((meta as any).engine_version || '1'),
    },
    envelope_version: '1',
  };
}

/**
 * Generate a premium asset.
 *
 * This is an *additive* method. It does not affect any existing NoesisClient methods.
 *
 * Two modes of operation:
 * 1. Server-seeded only (no llm): calls POST /api/v1/assets/generate and returns the
 *    manifest-shaped result exactly as before.
 * 2. Full local orchestration (llm + modeDoc or mode name): server call still happens
 *    (additive), then we fetch real seeds via calculate(), map to SelemeneEngineOutput[],
 *    run the real IntegratedReadingOrchestrator from @noesis/witness-pipeline, and
 *    attach orchestrator_output.
 */
export async function generatePremiumAsset(
  this: any, // bound to NoesisClient instance (has .request + .calculate)
  input: PremiumAssetInput,
  llm?: LlmCall,
  modeDoc?: ParsedModeDoc | string,
): Promise<PremiumAssetResult> {
  // ALWAYS call the server endpoint first. This is the source of the result envelope
  // and keeps the additive contract identical for callers that do not pass llm.
  const serverResult = await this.request(
    '/api/v1/assets/generate',
    {
      method: 'POST',
      body: JSON.stringify({
        birth_data: input.birth_data,
        mode: input.mode,
        consciousness_level: input.consciousness_level ?? 3,
        options: input.options ?? {},
      }),
    },
  ) as PremiumAssetResult;

  // Full local orchestration path (additive only).
  if (llm) {
    const resolved: ParsedModeDoc = resolveModeDoc(modeDoc || input.mode);

    // Build proper engine seeds for the orchestrator.
    // Prefer live calculate() calls (gives real witness_prompt + result envelopes).
    // Fall back to a minimal reconstruction if calculate is not available on this.
    let seeds: SelemeneEngineOutput[] = [];

    if (typeof this.calculate === 'function') {
      const engineIds = Object.keys(resolved.frontmatter.engine_overlay_weights);
      const results = await Promise.all(
        engineIds.map(async (eid: string) => {
          try {
            const eo: EngineOutput = await this.calculate(eid, {
              birth_data: input.birth_data,
            });
            return engineOutputToSelemene(eo, input.consciousness_level ?? 3);
          } catch (e: unknown) {
            const msg = e instanceof Error ? e.message : String(e);
            return {
              engine_id: eid as SelemeneEngineOutput['engine_id'],
              result: { _error: msg },
              witness_prompt: '',
              consciousness_level: input.consciousness_level ?? 3,
              metadata: {
                calculation_time_ms: 0,
                backend: 'selemene',
                precision_achieved: 'standard',
                cached: false,
                timestamp: new Date().toISOString(),
                engine_version: '1',
              },
              envelope_version: '1',
            } as SelemeneEngineOutput;
          }
        }),
      );
      seeds = results;
    } else if (serverResult && Array.isArray((serverResult as any).passes)) {
      // Best-effort reconstruction from server-shaped passes (partial but keeps server-as-source spirit).
      seeds = (serverResult as any).passes.map((p: any, i: number) => ({
        engine_id: (p.id || `engine-${i}`) as SelemeneEngineOutput['engine_id'],
        result: safeJsonParse(p.output) ?? p.output,
        witness_prompt: '',
        consciousness_level: input.consciousness_level ?? 3,
        metadata: {
          calculation_time_ms: 0,
          backend: 'server',
          precision_achieved: 'standard',
          cached: false,
          timestamp: new Date().toISOString(),
          engine_version: '1',
        },
        envelope_version: '1',
      }));
    }

    const mod: any = await import('@noesis/witness-pipeline');
    const orchestrator: IntegratedReadingOrchestrator = new mod.IntegratedReadingOrchestrator({
      mode: resolved,
      llm,
    });

    const orchInput: OrchestratorInput = {
      subjectNames: [input.birth_data?.name || 'Subject'],
      engineResultsBySubject: [seeds],
      consciousnessLevel: input.consciousness_level ?? 3,
    };

    const orchOut: OrchestratorOutput = await orchestrator.run(orchInput);

    return {
      ...serverResult,
      orchestrator_output: orchOut,
    };
  }

  // Server-only path: return exactly what the server gave (no extra keys, no behavior change).
  return serverResult;
}

function safeJsonParse(s: string | undefined): unknown {
  if (!s) return undefined;
  try {
    return JSON.parse(s);
  } catch {
    return s;
  }
}
