/**
 * GH #1411: the API sends witness_prompts as contract objects. When an output
 * has no singular witness_prompt, the pipeline must fall back to the first
 * prompt's text, not to the object itself.
 */
import { describe, expect, it } from "vitest";
import type { EngineOutput } from "./index.js";
import { engineOutputToSelemene } from "./premium-assets.js";

describe("engineOutputToSelemene witness prompt fallback (GH #1411)", () => {
  it("uses the first contract prompt's text when witness_prompt is absent", () => {
    const output: EngineOutput = {
      engine_id: "sacred-geometry",
      result: {},
      witness_prompts: [
        { prompt: "What do you notice?", context: "Contemplating", themes: [] },
        { prompt: "Before you begin: rest.", context: "Meditation guidance" },
      ],
    };

    expect(engineOutputToSelemene(output, 0).witness_prompt).toBe("What do you notice?");
  });

  it("still prefers the singular witness_prompt when present", () => {
    const output: EngineOutput = {
      engine_id: "sacred-geometry",
      result: {},
      witness_prompt: "Singular",
      witness_prompts: [{ prompt: "From list" }],
    };

    expect(engineOutputToSelemene(output, 0).witness_prompt).toBe("Singular");
  });
});
