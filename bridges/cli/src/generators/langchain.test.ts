// @ts-nocheck Bun supplies the test runtime; production dependencies stay unchanged.
import { describe, expect, test } from "bun:test";
import { generateLangChainTools } from "./langchain.js";

const PUBLIC_IDS = [
  "biofield",
  "biorhythm",
  "enneagram",
  "face-reading",
  "gene-keys",
  "human-design",
  "i-ching",
  "nadabrahman",
  "numerology",
  "panchanga",
  "raaga",
  "sacred-geometry",
  "sigil-forge",
  "tarot",
  "transits",
  "vedic-clock",
  "vimshottari",
];

function spec() {
  const paths = Object.fromEntries(
    PUBLIC_IDS.map((id) => [
      `/api/v1/engines/${id}/calculate`,
      {
        post: {
          description: `Calculate ${id}`,
          requestBody: { content: { "application/json": { schema: { type: "object" } } } },
        },
      },
    ]),
  );
  paths["/api/v1/engines/tarot/validate"] = { post: { description: "unsupported validate" } };
  paths["/ts/engines/tarot/calculate"] = { post: { description: "internal sidecar" } };
  return { openapi: "3.0.3", info: { title: "test", version: "1" }, paths };
}

const config = {
  rustUrl: "https://rust.example",
  tsUrl: "http://sidecar.internal",
  frameworks: ["langchain"],
  outputDir: "./out",
};

describe("protected Rust LangChain generation", () => {
  test("emits exactly the public mirror set and no sidecar/validate tools", () => {
    const content = generateLangChainTools(spec(), config).files[0].content;
    for (const id of PUBLIC_IDS) expect(content).toContain(`engines_${id.replaceAll("-", "_")}_calculate`);
    expect(content).toContain("selemene_engines_raaga_calculate");
    expect(content).not.toContain("financial_biosensor");
    expect(content).not.toContain("biofield_capture");
    expect(content).not.toContain("/ts/");
    expect(content).not.toContain("TS_URL");
    expect(content).not.toContain("validate");
  });

  test("keeps API-key and bearer branches distinct", () => {
    const content = generateLangChainTools(spec(), config).files[0].content;
    expect(content).toContain('h["X-API-Key"] = API_KEY');
    expect(content).toContain('h["Authorization"] = f"Bearer {BEARER_TOKEN}"');
    expect(content).not.toContain('h["Authorization"] = f"Bearer {API_KEY}"');
    expect(content).toContain("SELEMENE_BEARER_TOKEN");
  });
});
