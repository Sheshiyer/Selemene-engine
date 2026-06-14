"use client";

/**
 * Wave-1 component preview — not shipped, used for 1:1 visual QA against
 * docs/design/biofield-web/03-witness-dyad-spec.png.
 */

import { WitnessDyad } from "@/components/WitnessDyad";

const mock = {
  aletheios:
    "I witness from the threshold of what is. Your field shows a quiet asymmetry on the left, the body holding a question it has not yet spoken. This is not disorder — it is attention pooling before it moves. Let it gather.",
  pichet:
    "I witness from the place of becoming. The coherence is rising along the lower register, slow and deliberate, like breath finding its floor. Structure is forming here. Trust the unhurried climb and let it set before you ask it to carry weight.",
  synthesis:
    "Where Aletheios sees a held question and Pichet sees structure forming, the field is doing both at once: it is composing itself in stillness. Nothing here needs fixing. The asymmetry is the body deciding, and the rising coherence is the decision taking root.",
  witnessQuestion: "What is the way mine sees that serves the whole?",
  enginesUsed: ["biofield", "panchanga", "human-design"],
  llmPowered: true,
};

const label: React.CSSProperties = {
  fontFamily: "var(--font-mono, monospace)",
  fontSize: "0.7rem",
  letterSpacing: "0.12em",
  textTransform: "uppercase",
  color: "#8A9BA8",
  textAlign: "center",
};

export default function WitnessDyadPreview() {
  return (
    <main
      style={{
        minHeight: "100dvh",
        background: "#070B1D",
        padding: "clamp(2rem, 5vw, 4rem)",
        display: "flex",
        flexDirection: "column",
        gap: "3rem",
        alignItems: "center",
      }}
    >
      <div style={{ width: "100%", display: "flex", flexDirection: "column", gap: "1rem", alignItems: "center" }}>
        <span style={label}>full · llm-powered</span>
        <WitnessDyad
          aletheios={mock.aletheios}
          pichet={mock.pichet}
          synthesis={mock.synthesis}
          witnessQuestion={mock.witnessQuestion}
          enginesUsed={mock.enginesUsed}
          llmPowered={mock.llmPowered}
        />
      </div>

      <div style={{ width: "100%", display: "flex", flexDirection: "column", gap: "1rem", alignItems: "center" }}>
        <span style={label}>loading · witnessing</span>
        <WitnessDyad loading />
      </div>

      <div style={{ width: "100%", display: "flex", flexDirection: "column", gap: "1rem", alignItems: "center" }}>
        <span style={label}>empty · awaiting capture</span>
        <WitnessDyad />
      </div>
    </main>
  );
}
