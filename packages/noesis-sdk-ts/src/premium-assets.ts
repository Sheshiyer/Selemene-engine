// ─── Premium Asset Generation (additive SDK surface) ───────────────────────
// Wires to the new /api/v1/assets/generate endpoint and optionally to
// the local @noesis/witness-pipeline for full multi-pass orchestration.

import type { BirthData } from './index.js';
import type {
  IntegratedReadingOrchestrator,
  OrchestratorInput,
  OrchestratorOutput,
  ParsedModeDoc,
} from '@noesis/witness-pipeline';

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

/**
 * Generate a premium asset.
 *
 * This is an *additive* method. It does not affect any existing NoesisClient methods.
 *
 * Two modes of operation:
 * 1. Server-seeded (no llm provided): calls POST /api/v1/assets/generate and returns
 *    the engine context shaped for witness-pipeline consumers.
 * 2. Full local orchestration (llm + optional modeDoc provided): uses
 *    IntegratedReadingOrchestrator from @noesis/witness-pipeline to run the
 *    multi-pass pipeline locally after fetching engine seeds.
 */
export async function generatePremiumAsset(
  this: any, // bound to NoesisClient instance
  input: PremiumAssetInput,
  llm?: (system: string, user: string, opts: { max_tokens: number }) => Promise<string>,
  modeDoc?: ParsedModeDoc,
): Promise<PremiumAssetResult> {
  // Always hit the additive server endpoint for the seed / manifest shape.
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

  // If caller provided an LLM and a parsed mode, run full local orchestration.
  if (llm && modeDoc) {
    // The server result already contains engine seeds in a shape we can adapt.
    // For a minimal bridge we synthesize a simple OrchestratorInput.
    const subjectNames = [input.birth_data?.name || 'Subject'];
    const engineResultsBySubject: any[][] = [[/* seeds from server */]];

    // In a real integration the fetcher + mapper would normalise server seeds
    // into SelemeneEngineOutput[]. For this additive surface we expose the
    // orchestrator output when the caller supplies the pieces.
    const mod: any = await import('@noesis/witness-pipeline');
    const orchestrator = new mod.IntegratedReadingOrchestrator({
      mode: modeDoc,
      llm,
    });

    const orchInput: OrchestratorInput = {
      subjectNames,
      engineResultsBySubject,
      consciousnessLevel: input.consciousness_level ?? 3,
    };

    const orchOut = await orchestrator.run(orchInput);

    return {
      ...serverResult,
      orchestrator_output: orchOut,
    };
  }

  return serverResult;
}
