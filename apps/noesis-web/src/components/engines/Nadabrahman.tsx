"use client";

import { useState, useCallback } from "react";
import GenericEngineView from "./GenericEngineView";
import { MELAKARTAS, getRaagaPlayer } from "@/lib/raaga";

interface NadabrahmanProps {
  result: Record<string, unknown>;
}

const s = {
  grid: { display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(180px, 1fr))", gap: "0.75rem" },
  cell: { background: "var(--field)", borderRadius: "var(--radius)", padding: "0.75rem", display: "flex", flexDirection: "column" as const, gap: "0.25rem" },
  label: { fontSize: "0.7rem", color: "var(--text-muted)", textTransform: "uppercase" as const, letterSpacing: "0.06em", fontWeight: 600 },
  value: { fontSize: "1rem", fontWeight: 600, color: "var(--gold)" },
  sub: { fontSize: "0.8rem", color: "var(--text-muted)", fontFamily: "'IBM Plex Mono', monospace" },
  mantraBox: { padding: "1rem", background: "var(--gold-soft)", border: "1px solid var(--line-gold)", borderRadius: "var(--radius)", textAlign: "center" as const },
  mantra: { fontSize: "1.5rem", fontWeight: 700, color: "var(--gold)", fontFamily: "'Exo 2', sans-serif", letterSpacing: "0.1em" },
  tag: { display: "inline-flex", alignItems: "center", gap: "0.4rem", padding: "0.3rem 0.5rem 0.3rem 0.6rem", borderRadius: 4, background: "var(--field)", color: "var(--text)", fontSize: "0.8rem", marginRight: "0.3rem", marginBottom: "0.3rem", border: "1px solid var(--line)" },
  playBtn: { background: "var(--gold-soft)", border: "1px solid var(--line-gold)", color: "var(--gold)", borderRadius: "50%", width: 24, height: 24, cursor: "pointer", fontSize: "0.7rem", display: "inline-flex", alignItems: "center", justifyContent: "center", padding: 0 },
  playBtnActive: { background: "var(--gold)", color: "var(--bg)" },
  audioBar: { display: "flex", alignItems: "center", gap: "0.5rem", padding: "0.5rem 0.75rem", background: "var(--field)", borderRadius: "var(--radius)", fontSize: "0.8rem", color: "var(--text-muted)", border: "1px solid var(--line)" },
  stopBtn: { marginLeft: "auto", background: "transparent", border: "1px solid var(--line)", color: "var(--text-muted)", borderRadius: 4, padding: "0.2rem 0.6rem", cursor: "pointer", fontSize: "0.75rem" },
};

function str(v: unknown): string { return v == null ? "—" : String(v); }

/** Resolve any recommendation shape to a melakarta number, or null. */
function resolveMelakartaNum(r: unknown): { num: number; name: string } | null {
  if (typeof r === "number" && r >= 1 && r <= 72) {
    const m = MELAKARTAS[r - 1];
    return { num: r, name: m.name };
  }
  if (typeof r === "object" && r !== null) {
    const obj = r as Record<string, unknown>;
    const explicit = obj.raga_number ?? obj.melakarta ?? obj.num;
    if (typeof explicit === "number" && explicit >= 1 && explicit <= 72) {
      return { num: explicit, name: String(obj.raga_name ?? obj.name ?? MELAKARTAS[explicit - 1].name) };
    }
    const name = obj.raga_name ?? obj.name;
    if (typeof name === "string") {
      const found = matchByName(name);
      if (found) return found;
    }
  }
  if (typeof r === "string") return matchByName(r);
  return null;
}

function matchByName(s: string): { num: number; name: string } | null {
  const norm = s.toLowerCase().replace(/[^a-z]/g, "");
  if (!norm) return null;
  const hit = MELAKARTAS.find((m) => m.name.toLowerCase().replace(/[^a-z]/g, "").includes(norm));
  return hit ? { num: hit.num, name: hit.name } : null;
}

function getDisplayLabel(r: unknown): string {
  if (typeof r === "string") return r;
  if (typeof r === "number") return `#${r}`;
  if (typeof r === "object" && r !== null) {
    const obj = r as Record<string, unknown>;
    return String(obj.raga_name ?? obj.name ?? obj.raga_number ?? JSON.stringify(r));
  }
  return String(r);
}

export default function Nadabrahman({ result }: NadabrahmanProps) {
  const chakraFreq = result.chakra_frequency as Record<string, unknown> | undefined;
  const dosha = result.dosha_recommendation as string | undefined;
  const rasa = result.rasa_mapping as Record<string, unknown> | undefined;
  const rawRecs = (result.recommendations ?? []) as unknown[];
  const mantra = (chakraFreq?.mantra as string | undefined) ?? (result.mantra as string | undefined);
  const hz = chakraFreq?.frequency_hz ?? result.frequency_hz;
  const note = chakraFreq?.note ?? result.note;

  const [playingNum, setPlayingNum] = useState<number | null>(null);
  const [audioStatus, setAudioStatus] = useState<string | null>(null);

  const handlePlay = useCallback(async (num: number, name: string) => {
    try {
      setAudioStatus(`Loading audio…`);
      setPlayingNum(num);
      const rootHz = typeof hz === "number" ? hz : 220;
      await getRaagaPlayer().play(num, { rootHz, cps: 0.5, sound: "sine" });
      setAudioStatus(`▶ ${name} (#${num}) · just-intonation arohana ↑ avarohana ↓`);
    } catch (err) {
      setAudioStatus(`Audio error: ${(err as Error).message}`);
      setPlayingNum(null);
    }
  }, [hz]);

  const handleStop = useCallback(async () => {
    try { await getRaagaPlayer().stop(); } catch { /* ignore */ }
    setPlayingNum(null);
    setAudioStatus("Stopped");
  }, []);

  if (!chakraFreq && !mantra && !dosha) return <GenericEngineView result={result} />;

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "1rem" }}>
      {mantra != null && (
        <div style={s.mantraBox}>
          <div style={s.mantra}>{mantra}</div>
          {note != null && <div style={{ fontSize: "0.8rem", color: "var(--text-muted)", marginTop: "0.5rem" }}>Note: {str(note)}{hz != null ? ` · ${str(hz)} Hz` : ""}</div>}
        </div>
      )}

      <div style={s.grid}>
        {chakraFreq?.chakra != null && (
          <div style={s.cell}>
            <span style={s.label}>Chakra</span>
            <span style={s.value}>{str(chakraFreq.chakra)}</span>
            {chakraFreq.element != null && <span style={s.sub}>{str(chakraFreq.element)}</span>}
          </div>
        )}
        {dosha != null && (
          <div style={s.cell}>
            <span style={s.label}>Dosha</span>
            <span style={s.value}>{dosha}</span>
          </div>
        )}
        {rasa != null && (
          <div style={s.cell}>
            <span style={s.label}>Rasa</span>
            <span style={s.value}>{str(rasa.rasa ?? rasa.name)}</span>
            {rasa.emotion != null && <span style={s.sub}>{str(rasa.emotion)}</span>}
          </div>
        )}
      </div>

      {rawRecs.length > 0 && (
        <div style={{ display: "flex", flexDirection: "column" as const, gap: "0.5rem" }}>
          <span style={s.label}>Sound Practices</span>
          <div style={{ display: "flex", flexWrap: "wrap" as const }}>
            {rawRecs.slice(0, 6).map((r, i) => {
              const resolved = resolveMelakartaNum(r);
              const label = getDisplayLabel(r);
              const isActive = resolved != null && playingNum === resolved.num;
              return (
                <span key={i} style={s.tag}>
                  {label}
                  {resolved && (
                    <button
                      type="button"
                      style={{ ...s.playBtn, ...(isActive ? s.playBtnActive : {}) }}
                      onClick={() => handlePlay(resolved.num, resolved.name)}
                      title={`Play ${resolved.name} in just-intonation shrutis`}
                      aria-label={`Play ${resolved.name}`}
                    >
                      {isActive ? "♪" : "▶"}
                    </button>
                  )}
                </span>
              );
            })}
          </div>
        </div>
      )}

      {audioStatus && (
        <div style={s.audioBar}>
          <span>{audioStatus}</span>
          <button type="button" style={s.stopBtn} onClick={handleStop}>■ Stop</button>
        </div>
      )}
    </div>
  );
}
