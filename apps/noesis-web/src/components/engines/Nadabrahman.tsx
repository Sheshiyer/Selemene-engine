"use client";

import { useState, useCallback } from "react";
import GenericEngineView from "./GenericEngineView";
import { MELAKARTAS, getRaagaPlayer, type PlayOptions } from "@/lib/raaga";

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
  v2Bar: { display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(140px, 1fr))", gap: "0.5rem", padding: "0.5rem 0.75rem", background: "var(--gold-soft)", border: "1px solid var(--line-gold)", borderRadius: "var(--radius)" },
  v2Ctrl: { display: "flex", flexDirection: "column" as const, gap: "0.2rem", fontSize: "0.75rem" },
  v2Select: { background: "var(--field)", color: "var(--text)", border: "1px solid var(--line)", borderRadius: 4, padding: "0.25rem", fontSize: "0.8rem" },
  v2Toggle: { display: "flex", alignItems: "center", gap: "0.3rem", cursor: "pointer", fontSize: "0.75rem" },
  dlBtn: { background: "transparent", border: "1px solid var(--line)", color: "var(--text-muted)", borderRadius: 4, padding: "0.2rem 0.6rem", cursor: "pointer", fontSize: "0.75rem", marginLeft: "0.4rem" },
};

function str(v: unknown): string { return v == null ? "—" : String(v); }

interface ResolvedRaga { num: number; name: string; }

/** Resolve any recommendation shape to a melakarta. */
function resolveMelakartaNum(r: unknown): ResolvedRaga | null {
  if (typeof r === "number" && r >= 1 && r <= 72) {
    return { num: r, name: MELAKARTAS[r - 1].name };
  }
  if (typeof r === "object" && r !== null) {
    const obj = r as Record<string, unknown>;
    const explicit = obj.raga_number ?? obj.melakarta ?? obj.num;
    if (typeof explicit === "number" && explicit >= 1 && explicit <= 72) {
      return { num: explicit, name: String(obj.raga_name ?? obj.name ?? MELAKARTAS[explicit - 1].name) };
    }
    const name = obj.raga_name ?? obj.name;
    if (typeof name === "string") return matchByName(name);
  }
  if (typeof r === "string") return matchByName(r);
  return null;
}

function matchByName(s: string): ResolvedRaga | null {
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

type V2Timbre = 'sine' | 'sitar' | 'bansuri' | 'sarangi';
type V2GamakaKind = 'none' | 'kampita' | 'andolana' | 'kurula' | 'nokku' | 'sphurita';
type V2Tala = '' | 'adi' | 'rupakam' | 'misra-chapu' | 'khanda-chapu' | 'tisra-eka' | 'jhampa';
type V2Breath = '' | 'box-4' | 'calming-4-7-8' | 'bhastrika' | 'ujjayi' | 'brahmari' | 'shitali';

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

  // V2 control state
  const [v2On, setV2On] = useState(false);
  const [timbre, setTimbre] = useState<V2Timbre>('sitar');
  const [gamaka, setGamaka] = useState<V2GamakaKind>('kampita');
  const [tala, setTala] = useState<V2Tala>('adi');
  const [breath, setBreath] = useState<V2Breath>('');

  const buildPlayOptions = useCallback((): PlayOptions => {
    const opts: PlayOptions = {
      rootHz: typeof hz === "number" ? hz : 220,
      cps: 0.5,
    };
    if (v2On) {
      opts.v2 = true;
      opts.timbre = timbre;
      opts.defaultGamaka = { kind: gamaka } as PlayOptions['defaultGamaka'];
      if (tala) opts.tala = tala;
      if (breath) opts.breath = breath;
    } else {
      opts.sound = 'sine';
    }
    return opts;
  }, [hz, v2On, timbre, gamaka, tala, breath]);

  const handlePlay = useCallback(async (num: number, name: string) => {
    try {
      setAudioStatus(`Loading audio…`);
      setPlayingNum(num);
      await getRaagaPlayer().play(num, buildPlayOptions());
      const v2Tag = v2On ? `v2 · ${timbre} · ${gamaka}${tala ? ` · ${tala}` : ''}${breath ? ` · ${breath}` : ''}` : 'v1 sine';
      setAudioStatus(`▶ ${name} (#${num}) · ${v2Tag}`);
    } catch (err) {
      setAudioStatus(`Audio error: ${(err as Error).message}`);
      setPlayingNum(null);
    }
  }, [buildPlayOptions, v2On, timbre, gamaka, tala, breath]);

  const handleStop = useCallback(async () => {
    try { await getRaagaPlayer().stop(); } catch { /* ignore */ }
    setPlayingNum(null);
    setAudioStatus("Stopped");
  }, []);

  const handleDownload = useCallback(async (num: number, name: string) => {
    try {
      setAudioStatus(`⏳ Rendering ${name}…`);
      const blobUrl = await getRaagaPlayer().renderWav(num, buildPlayOptions());
      if (!blobUrl) {
        setAudioStatus("OfflineAudioContext unavailable in this browser");
        return;
      }
      const a = document.createElement('a');
      a.href = blobUrl;
      a.download = `${name}-${v2On ? timbre : 'sine'}.wav`;
      a.click();
      URL.revokeObjectURL(blobUrl);
      setAudioStatus(`⬇ Downloaded ${name}.wav`);
    } catch (err) {
      setAudioStatus(`Render error: ${(err as Error).message}`);
    }
  }, [buildPlayOptions, v2On, timbre]);

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
                    <>
                      <button
                        type="button"
                        style={{ ...s.playBtn, ...(isActive ? s.playBtnActive : {}) }}
                        onClick={() => handlePlay(resolved.num, resolved.name)}
                        title={`Play ${resolved.name}`}
                        aria-label={`Play ${resolved.name}`}
                      >
                        {isActive ? "♪" : "▶"}
                      </button>
                      <button
                        type="button"
                        style={s.dlBtn}
                        onClick={() => handleDownload(resolved.num, resolved.name)}
                        title={`Download ${resolved.name} as WAV`}
                        aria-label={`Download ${resolved.name}`}
                      >
                        ⬇ WAV
                      </button>
                    </>
                  )}
                </span>
              );
            })}
          </div>
        </div>
      )}

      {/* V2 controls — collapsed when v2 off */}
      <label style={s.v2Toggle}>
        <input type="checkbox" checked={v2On} onChange={(e) => setV2On(e.target.checked)} />
        <span style={{ color: v2On ? 'var(--gold)' : 'var(--text-muted)' }}>
          V2 audio: gamakas · timbres · tala · breath
        </span>
      </label>

      {v2On && (
        <div style={s.v2Bar}>
          <div style={s.v2Ctrl}>
            <span style={s.label}>Timbre</span>
            <select style={s.v2Select} value={timbre} onChange={(e) => setTimbre(e.target.value as V2Timbre)}>
              <option value="sine">Sine (V1)</option>
              <option value="sitar">Sitar</option>
              <option value="bansuri">Bansuri</option>
              <option value="sarangi">Sarangi</option>
            </select>
          </div>
          <div style={s.v2Ctrl}>
            <span style={s.label}>Gamaka</span>
            <select style={s.v2Select} value={gamaka} onChange={(e) => setGamaka(e.target.value as V2GamakaKind)}>
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
            <select style={s.v2Select} value={tala} onChange={(e) => setTala(e.target.value as V2Tala)}>
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
            <select style={s.v2Select} value={breath} onChange={(e) => setBreath(e.target.value as V2Breath)}>
              <option value="">Default cps</option>
              <option value="box-4">Box 4-4-4-4</option>
              <option value="calming-4-7-8">Calm 4-7-8</option>
              <option value="bhastrika">Bhastrika</option>
              <option value="ujjayi">Ujjāyī</option>
              <option value="brahmari">Bhrāmarī</option>
              <option value="shitali">Śītalī</option>
            </select>
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
