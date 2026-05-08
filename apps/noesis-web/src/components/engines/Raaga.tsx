"use client";

import { useState, useCallback } from "react";
import GenericEngineView from "./GenericEngineView";
import { getRaagaPlayer, MELAKARTAS } from "@/lib/raaga";

interface RaagaProps {
  result: Record<string, unknown>;
}

const s = {
  grid: { display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(170px, 1fr))", gap: "0.75rem" },
  cell: { background: "var(--field)", borderRadius: "var(--radius)", padding: "0.75rem", display: "flex", flexDirection: "column" as const, gap: "0.25rem" },
  label: { fontSize: "0.7rem", color: "var(--text-muted)", textTransform: "uppercase" as const, letterSpacing: "0.06em", fontWeight: 600 },
  value: { fontSize: "1rem", fontWeight: 600, color: "var(--gold)" },
  sub: { fontSize: "0.8rem", color: "var(--text-muted)", fontFamily: "'IBM Plex Mono', monospace" },
  section: { display: "flex", flexDirection: "column" as const, gap: "0.5rem" },
  sectionTitle: { fontSize: "0.75rem", color: "var(--text-muted)", textTransform: "uppercase" as const, letterSpacing: "0.07em", fontWeight: 600 },
  swaraGrid: { display: "grid", gridTemplateColumns: "repeat(8, 1fr)", gap: "0.375rem" },
  swaraCell: { background: "var(--field)", borderRadius: "calc(var(--radius) / 1.5)", padding: "0.5rem 0.25rem", textAlign: "center" as const, display: "flex", flexDirection: "column" as const, gap: "0.15rem" },
  swaraName: { fontSize: "0.75rem", fontWeight: 700, color: "var(--gold)" },
  swaraRatio: { fontSize: "0.65rem", color: "var(--text-muted)", fontFamily: "'IBM Plex Mono', monospace" },
  swaraHz: { fontSize: "0.65rem", color: "var(--text-muted)" },
  audioBar: { display: "flex", alignItems: "center", gap: "0.75rem", padding: "0.6rem 0.9rem", background: "var(--field)", borderRadius: "var(--radius)", border: "1px solid var(--line)" },
  playBtn: { background: "var(--gold-soft)", border: "1px solid var(--line-gold)", color: "var(--gold)", borderRadius: 6, padding: "0.35rem 0.9rem", cursor: "pointer", fontSize: "0.8rem", fontWeight: 600, display: "flex", alignItems: "center", gap: "0.4rem" },
  playBtnActive: { background: "var(--gold)", color: "var(--bg)" },
  stopBtn: { marginLeft: "auto", background: "transparent", border: "1px solid var(--line)", color: "var(--text-muted)", borderRadius: 4, padding: "0.25rem 0.7rem", cursor: "pointer", fontSize: "0.75rem" },
  statusText: { fontSize: "0.8rem", color: "var(--text-muted)", fontFamily: "'IBM Plex Mono', monospace" },
  tag: { display: "inline-flex", padding: "0.2rem 0.6rem", borderRadius: 4, background: "var(--field)", border: "1px solid var(--line)", color: "var(--text)", fontSize: "0.75rem", marginRight: "0.3rem", marginBottom: "0.3rem" },
  tagGold: { borderColor: "var(--line-gold)", color: "var(--gold)", background: "var(--gold-soft)" },
  praharBox: { padding: "0.75rem 1rem", background: "var(--gold-soft)", border: "1px solid var(--line-gold)", borderRadius: "var(--radius)", display: "flex", alignItems: "center", gap: "0.75rem" },
  praharLabel: { fontSize: "0.75rem", color: "var(--text-muted)", textTransform: "uppercase" as const, letterSpacing: "0.06em" },
  praharValue: { fontSize: "1rem", fontWeight: 700, color: "var(--gold)" },
};

function str(v: unknown): string { return v == null ? "—" : String(v); }
function fmt(n: number, d = 4): string { return n.toFixed(d); }

interface SwaraEntry {
  swara: string;
  ratio_decimal: number;
  hz: number;
}

interface Melakarta {
  num: number;
  name: string;
  chakra: number;
  ma_type: string;
}

interface PraharInfo {
  label: string;
  is_recommended_time: boolean;
}

export default function Raaga({ result }: RaagaProps) {
  const melakarta = result.melakarta as Melakarta | undefined;
  const swaras = (result.swaras ?? []) as SwaraEntry[];
  const rootHz = (result.root_hz as number | undefined) ?? 220;
  const prahar = result.prahar as PraharInfo | undefined;
  const doshaAffinities = result.dosha_affinities as Record<string, boolean> | undefined;
  const alternates = (result.alternate_ragas ?? []) as Array<{ num: number; name: string }>;

  const [isPlaying, setIsPlaying] = useState(false);
  const [audioStatus, setAudioStatus] = useState<string | null>(null);

  const handlePlay = useCallback(async () => {
    if (!melakarta) return;
    try {
      setAudioStatus("Loading audio engine…");
      const player = getRaagaPlayer();
      await player.play(melakarta.num, {
        rootHz,
        direction: "both",
        sound: "sine",
        cps: 0.4,
      });
      setIsPlaying(true);
      setAudioStatus(`▶ Playing ${melakarta.name} (Sa = ${rootHz}Hz)`);
    } catch (err) {
      setAudioStatus(`Error: ${err instanceof Error ? err.message : String(err)}`);
      setIsPlaying(false);
    }
  }, [melakarta, rootHz]);

  const handleStop = useCallback(async () => {
    try {
      const player = getRaagaPlayer();
      await player.stop();
    } catch (_) {
      // ignore
    }
    setIsPlaying(false);
    setAudioStatus(null);
  }, []);

  if (!melakarta) return <GenericEngineView result={result} />;

  const doshaLabels = Object.entries(doshaAffinities ?? {})
    .filter(([, v]) => v)
    .map(([k]) => k);

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "1.25rem" }}>

      {/* Header info */}
      <div style={s.grid}>
        <div style={s.cell}>
          <span style={s.label}>Melakarta</span>
          <span style={s.value}>#{melakarta.num} — {melakarta.name}</span>
          <span style={s.sub}>Chakra {melakarta.chakra} · {melakarta.ma_type === "prati" ? "Tivra Ma (M2)" : "Shuddha Ma (M1)"}</span>
        </div>
        <div style={s.cell}>
          <span style={s.label}>Root frequency</span>
          <span style={s.value}>{rootHz} Hz (Sa)</span>
          <span style={s.sub}>A3 concert pitch</span>
        </div>
        {prahar && (
          <div style={s.praharBox}>
            <div style={{ display: "flex", flexDirection: "column" as const }}>
              <span style={s.praharLabel}>Current Prahar</span>
              <span style={s.praharValue}>{prahar.label}</span>
              <span style={{ ...s.sub, fontSize: "0.7rem" }}>
                {prahar.is_recommended_time ? "✓ Recommended now" : "Not peak time"}
              </span>
            </div>
          </div>
        )}
        {doshaLabels.length > 0 && (
          <div style={s.cell}>
            <span style={s.label}>Dosha affinity</span>
            <div>
              {doshaLabels.map((d) => (
                <span key={d} style={{ ...s.tag, ...s.tagGold }}>{d}</span>
              ))}
            </div>
          </div>
        )}
      </div>

      {/* Arohana swara grid */}
      {swaras.length > 0 && (
        <div style={s.section}>
          <span style={s.sectionTitle}>Arohana · Ascending ({swaras.length} swaras)</span>
          <div style={s.swaraGrid}>
            {swaras.map((sw, i) => (
              <div key={i} style={s.swaraCell}>
                <span style={s.swaraName}>{sw.swara}</span>
                <span style={s.swaraRatio}>{fmt(sw.ratio_decimal)}</span>
                <span style={s.swaraHz}>{sw.hz}Hz</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Strudel audio playback */}
      <div style={s.section}>
        <span style={s.sectionTitle}>Sound · Strudel (just intonation)</span>
        <div style={s.audioBar}>
          <button
            style={{ ...s.playBtn, ...(isPlaying ? s.playBtnActive : {}) }}
            onClick={isPlaying ? handleStop : handlePlay}
            title={isPlaying ? "Stop" : `Play ${melakarta.name} arohana–avarohana`}
          >
            {isPlaying ? "⏹ Stop" : "▶ Play"}
          </button>
          {audioStatus && <span style={s.statusText}>{audioStatus}</span>}
          {isPlaying && (
            <button style={s.stopBtn} onClick={handleStop}>Stop</button>
          )}
        </div>
      </div>

      {/* Alternate ragas */}
      {alternates.length > 0 && (
        <div style={s.section}>
          <span style={s.sectionTitle}>Alternate ragas (same affinity)</span>
          <div>
            {alternates.map((a) => (
              <span key={a.num} style={s.tag}>#{a.num} {a.name}</span>
            ))}
          </div>
        </div>
      )}

    </div>
  );
}
