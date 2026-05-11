"use client";

import { useState, useCallback } from "react";
import GenericEngineView from "./GenericEngineView";
import { getRaagaPlayer, type PlayOptions } from "@/lib/raaga";
import { GAMAKA_DEFAULTS, type GamakaKind } from "@/lib/raaga/v2/gamakas/types";
import type { TalaName } from "@/lib/raaga/v2/talas/types";
import type { BreathName } from "@/lib/raaga/v2/breaths/data";
import type { Timbre } from "@/lib/raaga/v2/samples/timbres";

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
  swaraCellMuted: { background: "var(--bg)", opacity: 0.7, borderRadius: "calc(var(--radius) / 1.5)", padding: "0.5rem 0.25rem", textAlign: "center" as const, display: "flex", flexDirection: "column" as const, gap: "0.15rem" },
  swaraName: { fontSize: "0.75rem", fontWeight: 700, color: "var(--gold)" },
  swaraHz: { fontSize: "0.65rem", color: "var(--text-muted)" },
  audioBar: { display: "flex", alignItems: "center", gap: "0.75rem", padding: "0.6rem 0.9rem", background: "var(--field)", borderRadius: "var(--radius)", border: "1px solid var(--line)" },
  playBtn: { background: "var(--gold-soft)", border: "1px solid var(--line-gold)", color: "var(--gold)", borderRadius: 6, padding: "0.35rem 0.9rem", cursor: "pointer", fontSize: "0.8rem", fontWeight: 600, display: "flex", alignItems: "center", gap: "0.4rem" },
  playBtnActive: { background: "var(--gold)", color: "var(--bg)" },
  stopBtn: { background: "transparent", border: "1px solid var(--line)", color: "var(--text-muted)", borderRadius: 4, padding: "0.25rem 0.7rem", cursor: "pointer", fontSize: "0.75rem" },
  dlBtn: { background: "transparent", border: "1px solid var(--line)", color: "var(--text-muted)", borderRadius: 4, padding: "0.2rem 0.6rem", cursor: "pointer", fontSize: "0.75rem" },
  statusText: { fontSize: "0.8rem", color: "var(--text-muted)", fontFamily: "'IBM Plex Mono', monospace", flex: 1 },
  tag: { display: "inline-flex", padding: "0.2rem 0.6rem", borderRadius: 4, background: "var(--field)", border: "1px solid var(--line)", color: "var(--text)", fontSize: "0.75rem", marginRight: "0.3rem", marginBottom: "0.3rem" },
  tagGold: { borderColor: "var(--line-gold)", color: "var(--gold)", background: "var(--gold-soft)" },
  praharBox: { padding: "0.75rem 1rem", background: "var(--gold-soft)", border: "1px solid var(--line-gold)", borderRadius: "var(--radius)", display: "flex", alignItems: "center", gap: "0.75rem" },
  v2Toggle: { display: "flex", alignItems: "center", gap: "0.3rem", cursor: "pointer", fontSize: "0.8rem", userSelect: "none" as const },
  v2Bar: { display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(140px, 1fr))", gap: "0.5rem", padding: "0.75rem", background: "var(--gold-soft)", border: "1px solid var(--line-gold)", borderRadius: "var(--radius)" },
  v2Ctrl: { display: "flex", flexDirection: "column" as const, gap: "0.2rem" },
  v2Select: { background: "var(--field)", color: "var(--text)", border: "1px solid var(--line)", borderRadius: 4, padding: "0.25rem", fontSize: "0.8rem" },
};

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
  num?: number;
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
  const mood = result.mood as string | undefined;
  const rasa = result.rasa as string | undefined;
  const vadi = result.vadi as string | undefined;
  const samvadi = result.samvadi as string | undefined;

  // Avaroha = arohana reversed (standard melakarta rule)
  const avaroha: SwaraEntry[] = [...swaras].reverse().map((sw, i, arr) => ({
    ...sw,
    swara: i === 0 ? "Sa'" : i === arr.length - 1 ? "Sa" : sw.swara,
  }));

  const [isPlaying, setIsPlaying] = useState(false);
  const [audioStatus, setAudioStatus] = useState<string | null>(null);

  // V2 control state
  const [v2On, setV2On] = useState(false);
  const [timbre, setTimbre] = useState<Timbre>("sitar");
  const [gamaka, setGamaka] = useState<GamakaKind>("kampita");
  const [tala, setTala] = useState<TalaName | "">("");
  const [breath, setBreath] = useState<BreathName | "">("");

  const buildPlayOptions = useCallback((): PlayOptions => {
    const base: PlayOptions = { rootHz, direction: "both", cps: 0.4 };
    if (v2On) {
      return {
        ...base,
        v2: true,
        timbre,
        defaultGamaka: GAMAKA_DEFAULTS[gamaka],
        ...(tala ? { tala: tala as TalaName } : {}),
        ...(breath ? { breath: breath as BreathName } : {}),
      };
    }
    return { ...base, sound: "sine" };
  }, [rootHz, v2On, timbre, gamaka, tala, breath]);

  const handlePlay = useCallback(async () => {
    if (!melakarta) return;
    try {
      setAudioStatus("Loading audio engine…");
      await getRaagaPlayer().play(melakarta.num, buildPlayOptions());
      setIsPlaying(true);
      const tag = v2On
        ? `${timbre} · ${gamaka}${tala ? ` · ${tala}` : ""}${breath ? ` · ${breath}` : ""}`
        : "sine";
      setAudioStatus(`▶ ${melakarta.name} (Sa = ${rootHz}Hz) · ${tag}`);
    } catch (err) {
      setAudioStatus(`Error: ${err instanceof Error ? err.message : String(err)}`);
      setIsPlaying(false);
    }
  }, [melakarta, rootHz, buildPlayOptions, v2On, timbre, gamaka, tala, breath]);

  const handleStop = useCallback(async () => {
    try { await getRaagaPlayer().stop(); } catch { /* ignore */ }
    setIsPlaying(false);
    setAudioStatus(null);
  }, []);

  const handleDownload = useCallback(async () => {
    if (!melakarta) return;
    try {
      setAudioStatus(`⏳ Rendering ${melakarta.name}…`);
      const blobUrl = await getRaagaPlayer().renderWav(melakarta.num, buildPlayOptions());
      if (!blobUrl) { setAudioStatus("OfflineAudioContext unavailable"); return; }
      const a = document.createElement("a");
      a.href = blobUrl;
      a.download = `${melakarta.name}-${v2On ? timbre : "sine"}.wav`;
      a.click();
      URL.revokeObjectURL(blobUrl);
      setAudioStatus(`⬇ Downloaded ${melakarta.name}.wav`);
    } catch (err) {
      setAudioStatus(`Render error: ${err instanceof Error ? err.message : String(err)}`);
    }
  }, [melakarta, buildPlayOptions, v2On, timbre]);

  if (!melakarta) return <GenericEngineView result={result} />;

  const doshaLabels = Object.entries(doshaAffinities ?? {})
    .filter(([, v]) => v)
    .map(([k]) => k);

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "1.25rem" }}>

      {/* Header */}
      <div style={s.grid}>
        <div style={s.cell}>
          <span style={s.label}>Melakarta</span>
          <span style={s.value}>#{melakarta.num} — {melakarta.name}</span>
          <span style={s.sub}>Chakra {melakarta.chakra} · {melakarta.ma_type === "prati" ? "Tīvra Ma (M2)" : "Shuddha Ma (M1)"}</span>
        </div>
        <div style={s.cell}>
          <span style={s.label}>Root frequency</span>
          <span style={s.value}>{rootHz} Hz (Sa)</span>
          <span style={s.sub}>A3 concert pitch</span>
        </div>
        {prahar && (
          <div style={s.praharBox}>
            <div style={{ display: "flex", flexDirection: "column" as const }}>
              <span style={s.label}>Current Prahar</span>
              <span style={s.value}>{prahar.label}</span>
              <span style={{ ...s.sub, fontSize: "0.7rem" }}>
                {prahar.is_recommended_time ? "✓ Recommended now" : "Not peak time"}
              </span>
            </div>
          </div>
        )}
        {doshaLabels.length > 0 && (
          <div style={s.cell}>
            <span style={s.label}>Dosha affinity</span>
            <div>{doshaLabels.map((d) => <span key={d} style={{ ...s.tag, ...s.tagGold }}>{d}</span>)}</div>
          </div>
        )}
        {(vadi || samvadi) && (
          <div style={s.cell}>
            <span style={s.label}>Vadi · Samvadi</span>
            {vadi && <span style={s.value}>{vadi}{samvadi ? ` · ${samvadi}` : ""}</span>}
          </div>
        )}
        {(rasa || mood) && (
          <div style={s.cell}>
            <span style={s.label}>Rasa / Mood</span>
            <span style={s.value}>{rasa ?? mood}</span>
          </div>
        )}
      </div>

      {/* Arohana */}
      {swaras.length > 0 && (
        <div style={s.section}>
          <span style={s.sectionTitle}>Ārohana · Ascending</span>
          <div style={s.swaraGrid}>
            {swaras.map((sw, i) => (
              <div key={i} style={s.swaraCell}>
                <span style={s.swaraName}>{sw.swara}</span>
                <span style={s.swaraHz}>{sw.hz}Hz</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Avarohana */}
      {avaroha.length > 0 && (
        <div style={s.section}>
          <span style={s.sectionTitle}>Avarohana · Descending</span>
          <div style={s.swaraGrid}>
            {avaroha.map((sw, i) => (
              <div key={i} style={s.swaraCellMuted}>
                <span style={s.swaraName}>{sw.swara}</span>
                <span style={s.swaraHz}>{sw.hz}Hz</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* V2 audio controls */}
      <div style={s.section}>
        <span style={s.sectionTitle}>Sound · Strudel (just intonation)</span>

        <label style={s.v2Toggle}>
          <input type="checkbox" checked={v2On} onChange={(e) => setV2On(e.target.checked)} />
          <span style={{ color: v2On ? "var(--gold)" : "var(--text-muted)" }}>
            V2: gamakas · timbres · tala · breath
          </span>
        </label>

        {v2On && (
          <div style={s.v2Bar}>
            <div style={s.v2Ctrl}>
              <span style={s.label}>Timbre</span>
              <select style={s.v2Select} value={timbre} onChange={(e) => setTimbre(e.target.value as Timbre)}>
                <option value="sine">Sine (V1)</option>
                <option value="sitar">Sitar</option>
                <option value="bansuri">Bansuri</option>
                <option value="sarangi">Sarangi</option>
                <option value="tanpura">Tanpura</option>
                <option value="mridangam">Mridangam</option>
                <option value="pad">Pad (synth)</option>
                <option value="dronesynth">Drone synth</option>
                <option value="sawlead">Saw lead</option>
                <option value="squarepluck">Square pluck</option>
                <option value="supersawpad">Supersaw pad</option>
              </select>
            </div>
            <div style={s.v2Ctrl}>
              <span style={s.label}>Gamaka</span>
              <select style={s.v2Select} value={gamaka} onChange={(e) => setGamaka(e.target.value as GamakaKind)}>
                <option value="none">None</option>
                <option value="kampita">Kampita (vibrato)</option>
                <option value="andolana">Andolana (sway)</option>
                <option value="kurula">Kurula (slide)</option>
                <option value="nokku">Nokku (grace)</option>
                <option value="sphurita">Sphurita (bent attack)</option>
              </select>
            </div>
            <div style={s.v2Ctrl}>
              <span style={s.label}>Tala</span>
              <select style={s.v2Select} value={tala} onChange={(e) => setTala(e.target.value as TalaName | "")}>
                <option value="">Free</option>
                <option value="adi">Ādi (8)</option>
                <option value="rupakam">Rūpakam (6)</option>
                <option value="misra-chapu">Miśra Chāpu (7)</option>
                <option value="khanda-chapu">Khaṇḍa Chāpu (5)</option>
                <option value="tisra-eka">Tisra Ēkam (3)</option>
                <option value="jhampa">Miśra Jhampa (10)</option>
              </select>
            </div>
            <div style={s.v2Ctrl}>
              <span style={s.label}>Breath</span>
              <select style={s.v2Select} value={breath} onChange={(e) => setBreath(e.target.value as BreathName | "")}>
                <option value="">Default cps</option>
                <option value="box-4">Box 4-4-4-4</option>
                <option value="calming-4-7-8">Calm 4-7-8</option>
                <option value="coherence-6-0-6-0">Coherence 6-0-6-0</option>
                <option value="heart-coherence-5-5">Heart 5-5</option>
                <option value="bhastrika">Bhastrika</option>
                <option value="kapalabhati">Kapālabhāti</option>
                <option value="nadi-shodhana">Nāḍī Shodhana</option>
                <option value="ujjayi">Ujjāyī</option>
                <option value="brahmari">Bhrāmarī</option>
                <option value="shitali">Śītalī</option>
                <option value="dirgha">Dīrgha</option>
              </select>
            </div>
          </div>
        )}

        <div style={s.audioBar}>
          <button
            style={{ ...s.playBtn, ...(isPlaying ? s.playBtnActive : {}) }}
            onClick={isPlaying ? handleStop : handlePlay}
            title={isPlaying ? "Stop" : `Play ${melakarta.name} ārohana–avarohana`}
          >
            {isPlaying ? "⏹ Stop" : "▶ Play"}
          </button>
          {audioStatus && <span style={s.statusText}>{audioStatus}</span>}
          {!isPlaying && v2On && (
            <button style={s.dlBtn} onClick={handleDownload} title="Download as WAV">
              ⬇ WAV
            </button>
          )}
          {isPlaying && (
            <button style={s.stopBtn} onClick={handleStop}>■ Stop</button>
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

