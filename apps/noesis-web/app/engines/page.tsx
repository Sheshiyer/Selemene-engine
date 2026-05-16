"use client";

import { useState, useEffect, useCallback } from "react";
import { useRouter } from "next/navigation";
import NavBar from "@/components/NavBar";
import BirthDataForm from "@/components/BirthDataForm";
import WitnessLayer from "@/components/WitnessLayer";
import EngineCard from "@/components/EngineCard";
import CalculationLoader from "@/components/CalculationLoader";
import CompassSelector, {
  COMPASS_OPTIONS,
  type CompassMode,
} from "@/components/CompassSelector";
import EngineGrid, { type EngineGridItem } from "@/components/EngineGrid";
import { getApiKey, isAuthenticated } from "@/lib/auth";
import {
  executeWorkflow,
  getMe,
  getReadings,
  getReading,
  type BirthData,
  type WorkflowResponse,
  type EngineOutput,
} from "@/lib/api";

import Panchanga from "@/components/engines/Panchanga";
import HumanDesign from "@/components/engines/HumanDesign";
import GeneKeys from "@/components/engines/GeneKeys";
import Vimshottari from "@/components/engines/Vimshottari";
import Numerology from "@/components/engines/Numerology";
import Biorhythm from "@/components/engines/Biorhythm";
import Tarot from "@/components/engines/Tarot";
import IChing from "@/components/engines/IChing";
import Transits from "@/components/engines/Transits";
import BiofieldView from "@/components/engines/Biofield";
import VedicClock from "@/components/engines/VedicClock";
import SacredGeometry from "@/components/engines/SacredGeometry";
import SigilForge from "@/components/engines/SigilForge";
import Enneagram from "@/components/engines/Enneagram";
import Nadabrahman from "@/components/engines/Nadabrahman";
import FaceReading from "@/components/engines/FaceReading";
import RaagaView from "@/components/engines/Raaga";
import GenericEngineView from "@/components/engines/GenericEngineView";

/* ── Engine registry ─────────────────────────────────── */

const ENGINE_IDS = [
  "panchanga",
  "human-design",
  "gene-keys",
  "vimshottari",
  "numerology",
  "biorhythm",
  "vedic-clock",
  "transits",
  "biofield",
  "tarot",
  "i-ching",
  "sacred-geometry",
  "sigil-forge",
  "enneagram",
  "nadabrahman",
  "face-reading",
  "raaga",
] as const;

type EngineId = (typeof ENGINE_IDS)[number];

const ENGINE_META: Record<EngineId, EngineGridItem> = {
  panchanga: {
    id: "panchanga",
    label: "Panchanga",
    sigil: "☽",
    direction: "stabilize",
  },
  "human-design": {
    id: "human-design",
    label: "Human Design",
    sigil: "⬡",
    direction: "mutate",
  },
  "gene-keys": {
    id: "gene-keys",
    label: "Gene Keys",
    sigil: "✦",
    direction: "mutate",
  },
  vimshottari: {
    id: "vimshottari",
    label: "Vimshottari",
    sigil: "◌",
    direction: "stabilize",
  },
  numerology: {
    id: "numerology",
    label: "Numerology",
    sigil: "9",
    direction: "mutate",
  },
  biorhythm: {
    id: "biorhythm",
    label: "Biorhythm",
    sigil: "∞",
    direction: "heal",
  },
  "vedic-clock": {
    id: "vedic-clock",
    label: "Vedic Clock",
    sigil: "◷",
    direction: "heal",
  },
  transits: {
    id: "transits",
    label: "Transits",
    sigil: "☍",
    direction: "stabilize",
  },
  biofield: {
    id: "biofield",
    label: "Biofield",
    sigil: "◎",
    direction: "heal",
  },
  tarot: {
    id: "tarot",
    label: "Tarot",
    sigil: "▯",
    direction: "mutate",
  },
  "i-ching": {
    id: "i-ching",
    label: "I-Ching",
    sigil: "☷",
    direction: "mutate",
  },
  "sacred-geometry": {
    id: "sacred-geometry",
    label: "Sacred Geometry",
    sigil: "✺",
    direction: "create",
  },
  "sigil-forge": {
    id: "sigil-forge",
    label: "Sigil Forge",
    sigil: "⌁",
    direction: "create",
  },
  enneagram: {
    id: "enneagram",
    label: "Enneagram",
    sigil: "✶",
    direction: "mutate",
  },
  nadabrahman: {
    id: "nadabrahman",
    label: "Nadabrahman",
    sigil: "ॐ",
    direction: "create",
  },
  "face-reading": {
    id: "face-reading",
    label: "Face Reading",
    sigil: "◉",
    direction: "mutate",
  },
  raaga: {
    id: "raaga",
    label: "Raaga",
    sigil: "♪",
    direction: "create",
  },
};

const COMPASS_WORKFLOWS: Record<
  CompassMode,
  { label: string; workflowId: string; engineIds: EngineId[] }
> = {
  "full-spectrum": {
    label: "Full Spectrum",
    workflowId: "full-spectrum",
    engineIds: [...ENGINE_IDS],
  },
  stabilize: {
    label: "Stabilize",
    workflowId: "daily-practice",
    engineIds: ["panchanga", "vedic-clock", "biorhythm", "transits", "nadabrahman"],
  },
  heal: {
    label: "Heal",
    workflowId: "self-inquiry",
    engineIds: ["gene-keys", "enneagram", "face-reading", "biofield"],
  },
  create: {
    label: "Create",
    workflowId: "creative-expression",
    engineIds: ["sigil-forge", "sacred-geometry", "nadabrahman", "numerology", "raaga"],
  },
  mutate: {
    label: "Mutate",
    workflowId: "decision-support",
    engineIds: ["tarot", "i-ching", "human-design", "enneagram", "gene-keys"],
  },
};

function engineLabel(id: string): string {
  return ENGINE_META[id as EngineId]?.label ?? id
    .split("-")
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
    .join(" ");
}

function renderEngine(id: EngineId, result: Record<string, unknown>) {
  switch (id) {
    case "panchanga":
      return <Panchanga result={result} />;
    case "human-design":
      return <HumanDesign result={result} />;
    case "gene-keys":
      return <GeneKeys result={result} />;
    case "vimshottari":
      return <Vimshottari result={result} />;
    case "numerology":
      return <Numerology result={result} />;
    case "biorhythm":
      return <Biorhythm result={result} />;
    case "vedic-clock":
      return <VedicClock result={result} />;
    case "transits":
      return <Transits result={result} />;
    case "biofield":
      return <BiofieldView result={result} />;
    case "tarot":
      return <Tarot result={result} />;
    case "i-ching":
      return <IChing result={result} />;
    case "sacred-geometry":
      return <SacredGeometry result={result} />;
    case "sigil-forge":
      return <SigilForge result={result} />;
    case "enneagram":
      return <Enneagram result={result} />;
    case "nadabrahman":
      return <Nadabrahman result={result} />;
    case "face-reading":
      return <FaceReading result={result} />;
    case "raaga":
      return <RaagaView result={result} />;
    default:
      return <GenericEngineView result={result} />;
  }
}

/* ── DX-03 — Constellation background ───────────────── */

// Deterministic star field (no JS randomness — seeded via golden-angle hash)
const STARS = Array.from({ length: 72 }, (_, i) => ({
  cx: ((i * 137.508 + 17.3) % 100).toFixed(2),
  cy: ((i * 89.217 + 43.1) % 100).toFixed(2),
  r:  i % 7 === 0 ? "0.22" : i % 3 === 0 ? "0.15" : "0.09",
  op: (0.10 + (i % 8) * 0.04).toFixed(2),
}));

const LINES = Array.from({ length: 22 }, (_, i) => {
  const a = STARS[i * 3 % 72];
  const b = STARS[(i * 3 + 9) % 72];
  return { x1: a.cx, y1: a.cy, x2: b.cx, y2: b.cy };
});

function ConstellationBg() {
  return (
    <svg
      aria-hidden="true"
      className="constellation-bg"
      viewBox="0 0 100 100"
      preserveAspectRatio="xMidYMid slice"
    >
      {LINES.map((l, i) => (
        <line
          key={i}
          x1={l.x1} y1={l.y1} x2={l.x2} y2={l.y2}
          stroke="rgba(197,160,23,0.05)"
          strokeWidth="0.06"
        />
      ))}
      {STARS.map((star, i) => (
        <circle
          key={i}
          cx={star.cx} cy={star.cy}
          r={star.r}
          fill={i % 5 === 0 ? "rgba(16,181,167,0.7)" : "rgba(197,160,23,0.8)"}
          opacity={star.op}
        />
      ))}
    </svg>
  );
}

/* ── DX-26 — Awaiting Calculation empty state ───────── */

function EmptyState({ label, glyph }: { label: string; glyph: string }) {
  return (
    <div style={{
      padding: "4rem 2rem",
      border: "1px solid rgba(255, 255, 255, 0.08)",
      borderRadius: "10px",
      background: "#000000",
      display: "flex",
      flexDirection: "column",
      alignItems: "center",
      gap: "1.5rem",
      textAlign: "center",
      position: "relative",
      overflow: "hidden",
      animation: "deepOceanFadeIn 0.6s ease both",
    }}>
      {/* Atmospheric Deep Ocean orb — depth without shadows */}
      <div aria-hidden style={{
        position: "absolute",
        top: "50%", left: "50%",
        width: 320, height: 320,
        borderRadius: "50%",
        background: "linear-gradient(135deg, rgb(160, 224, 171) 0%, rgb(255, 172, 46) 50%, rgb(165, 45, 37) 100%)",
        animation: "deepOceanOrb 6s ease-in-out infinite",
        pointerEvents: "none",
      }} />

      {/* Glyph — etched on frosted surface */}
      <div style={{
        position: "relative",
        width: 64, height: 64,
        borderRadius: "50%",
        border: "1px solid rgba(255, 255, 255, 0.12)",
        background: "rgba(255, 255, 255, 0.03)",
        display: "flex", alignItems: "center", justifyContent: "center",
        backdropFilter: "blur(8px)",
        fontSize: "1.5rem",
        color: "rgba(255, 255, 255, 0.5)",
      }} aria-hidden>
        {glyph}
      </div>

      <span style={{
        position: "relative",
        fontFamily: "system-ui, -apple-system, sans-serif",
        fontSize: "0.6875rem",
        fontWeight: 400,
        textTransform: "uppercase",
        letterSpacing: "0.2em",
        color: "#ffffff",
      }}>
        Awaiting Calculation
      </span>

      <p style={{
        position: "relative",
        color: "#6d6d6d",
        fontSize: "0.9375rem",
        lineHeight: 1.65,
        maxWidth: 380,
        margin: 0,
        fontFamily: "system-ui, -apple-system, sans-serif",
      }}>
        Enter your birth data above and begin the{" "}
        <span style={{ color: "rgba(255, 255, 255, 0.45)" }}>{label}</span>{" "}
        ritual to reveal your cosmic blueprint.
      </p>
    </div>
  );
}

/* ── Styles ──────────────────────────────────────────── */

const s = {
  page: {
    flex: 1,
    display: "flex",
    flexDirection: "column" as const,
    background: "var(--bg)",
    minHeight: "100vh",
  },
  content: {
    maxWidth: 1100,
    width: "100%",
    margin: "0 auto",
    padding: "1.5rem",
    display: "flex",
    flexDirection: "column" as const,
    gap: "1.5rem",
  },
  heading: {
    fontFamily: "var(--font-display)",
    fontSize: "1.75rem",
    fontWeight: 800,
    color: "var(--signal)",
    letterSpacing: "0.06em",
  },
  intro: {
    maxWidth: 680,
    color: "var(--text-2)",
    fontSize: "0.95rem",
    lineHeight: 1.7,
    marginTop: "-0.75rem",
  },
  errorBox: {
    padding: "1rem",
    background: "rgba(239,107,115,0.1)",
    border: "1px solid var(--danger)",
    borderRadius: "var(--radius)",
    color: "var(--danger)",
    fontSize: "0.9rem",
  },
  meta: {
    display: "flex",
    flexWrap: "wrap" as const,
    gap: "1rem",
    fontSize: "0.8rem",
    color: "var(--text-muted)",
    fontFamily: "var(--font-mono)",
  },
  resultShell: {
    display: "flex",
    flexDirection: "column" as const,
    gap: "1rem",
  },
  synthesisButton: {
    width: "fit-content",
    padding: "0.625rem 0.875rem",
    borderRadius: "var(--r-sm)",
    border: "1px solid var(--line-mid)",
    background: "rgba(11,80,251,0.05)",
    color: "var(--text-2)",
    fontFamily: "var(--font-mono)",
    fontSize: "0.72rem",
    letterSpacing: "0.08em",
    textTransform: "uppercase" as const,
    transition: "border-color 0.18s, color 0.18s, box-shadow 0.18s",
  },
  synthesisButtonActive: {
    borderColor: "rgba(197,160,23,0.45)",
    color: "var(--signal)",
    boxShadow: "var(--glow-gold)",
  },
};

/* ── Page ────────────────────────────────────────────── */

const BIRTH_DATA_KEY = "noesis:lastBirthData";

function saveBirthDataLocally(data: Partial<BirthData>) {
  try { localStorage.setItem(BIRTH_DATA_KEY, JSON.stringify(data)); } catch { /* quota */ }
}

function loadBirthDataLocally(): Partial<BirthData> | undefined {
  try {
    const raw = localStorage.getItem(BIRTH_DATA_KEY);
    return raw ? JSON.parse(raw) : undefined;
  } catch { return undefined; }
}

export default function EnginesPage() {
  const router = useRouter();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [response, setResponse] = useState<WorkflowResponse | null>(null);
  const [activeTab, setActiveTab] = useState<string>("witness");
  const [selectedCompass, setSelectedCompass] =
    useState<CompassMode>("full-spectrum");
  const [lastBirthData, setLastBirthData] = useState<Partial<BirthData> | undefined>(undefined);
  const [formExpanded, setFormExpanded] = useState(false);

  useEffect(() => {
    if (!isAuthenticated()) {
      router.replace("/auth");
      return;
    }

    // Immediately restore birth data from localStorage so the chip shows
    // without waiting for the API round-trip.
    const cached = loadBirthDataLocally();
    if (cached?.date) setLastBirthData(cached);

    const key = getApiKey();
    if (!key) return;

    // Primary source: /users/me profile (has birth data stored in Supabase).
    // This is more reliable than readings since it's explicitly saved by the user.
    getMe(key)
      .then((profile) => {
        if (profile.birth_date) {
          const bd: Partial<BirthData> = {
            date: profile.birth_date,
            time: profile.birth_time ?? undefined,
            latitude: profile.birth_location?.lat,
            longitude: profile.birth_location?.lng,
            timezone: profile.timezone ?? undefined,
            name: profile.full_name ?? undefined,
          };
          setLastBirthData(bd);
          saveBirthDataLocally(bd as BirthData);
        }
      })
      .catch(() => { /* profile not available — use localStorage/readings fallback */ });

    // Secondary source: last reading's input_data.birth_data
    getReadings(key)
      .then(async (res) => {
        const latest = res.readings?.[0];
        if (!latest) return;
        if (latest.input_data) {
          const raw = latest.input_data as { birth_data?: BirthData | null } & Partial<BirthData>;
          const bd: Partial<BirthData> | null = raw.birth_data ?? (raw.date ? raw as Partial<BirthData> : null);
          // Only update if we don't already have profile data (profile wins)
          if (bd?.date) {
            setLastBirthData((prev) => prev?.date ? prev : bd);
            saveBirthDataLocally(bd as BirthData);
          }
        }
        if (latest.id) {
          try {
            const full = await getReading(latest.id, key);
            const workflowResult = full.result_data;
            if (workflowResult && (workflowResult.engine_outputs || workflowResult.engine_results)) {
              setResponse(workflowResult);
              setActiveTab("witness");
            }
          } catch { /* can't load full reading — chip still shows */ }
        }
      })
      .catch(() => { /* API down — localStorage fallback already applied */ });
  }, [router]);

  const engineMap = new Map<string, EngineOutput>();
  const outputsMap = response?.engine_outputs ?? response?.engine_results ?? {};
  for (const [id, o] of Object.entries(outputsMap)) {
    engineMap.set(id, o);
  }
  const selectedWorkflow = COMPASS_WORKFLOWS[selectedCompass];
  const selectedGridEngines = selectedWorkflow.engineIds.map((id) => ENGINE_META[id]);
  const selectedEngineLabels = selectedWorkflow.engineIds.map((id) =>
    ENGINE_META[id].label.toLowerCase(),
  );
  const availableEngineIds = new Set(engineMap.keys());

  const handleCompassSelect = (mode: CompassMode) => {
    setSelectedCompass(mode);
    setActiveTab("witness");
    setResponse(null);
    setError(null);
  };

  const handleSubmit = useCallback(async (birthData: BirthData) => {
    const key = getApiKey();
    if (!key) {
      setError("No API key found. Please enter your API key.");
      return;
    }
    setLoading(true);
    setError(null);
    setResponse(null);
    setActiveTab("witness");
    try {
      const res = await executeWorkflow(selectedWorkflow.workflowId, birthData, key);
      setResponse(res);
      setLastBirthData(birthData);
      saveBirthDataLocally(birthData);
      setFormExpanded(false);
    } catch (err: unknown) {
      if (err instanceof Error) {
        setError(err.message);
      } else {
        setError("An unknown error occurred.");
      }
    } finally {
      setLoading(false);
    }
  }, [selectedWorkflow.workflowId]);

  return (
    <div style={s.page}>
      <ConstellationBg />
      <NavBar />
      <main className="page-content" style={s.content}>
        <h1 style={s.heading}>{selectedWorkflow.label}</h1>
        <p style={s.intro}>
          Choose a compass direction before calculation. Full Spectrum keeps the
          complete 17-engine mandala; directional modes focus the workflow into
          a ritual arc.
        </p>
        <CompassSelector
          selected={selectedCompass}
          onSelect={handleCompassSelect}
        />

        {/* Collapse form when birth data is known (from DB or prior session) */}
        {(lastBirthData?.date && !formExpanded) ? (
          <div style={{
            display: "flex",
            alignItems: "center",
            gap: "0.75rem",
            padding: "0.65rem 1rem",
            border: "1px solid var(--line-mid)",
            borderRadius: "var(--r-sm)",
            background: "var(--surface-1)",
            fontSize: "0.8rem",
            fontFamily: "var(--font-mono)",
            color: "var(--text-2)",
            flexWrap: "wrap",
          }}>
            <span style={{ color: "var(--c-emerald)", marginRight: "0.2rem" }}>◈</span>
            {lastBirthData.name && <span style={{ color: "var(--text)", fontWeight: 600 }}>{lastBirthData.name}</span>}
            {lastBirthData.date && <span>{lastBirthData.date}</span>}
            {lastBirthData.timezone && <span style={{ color: "var(--muted)" }}>{lastBirthData.timezone}</span>}
            {lastBirthData.latitude != null && (
              <span style={{ color: "var(--muted)" }}>{Number(lastBirthData.latitude).toFixed(2)}, {Number(lastBirthData.longitude).toFixed(2)}</span>
            )}
            <div style={{ marginLeft: "auto", display: "flex", gap: "0.5rem", alignItems: "center" }}>
              {/* Show "Run Analysis" when no results are loaded yet (e.g. on fresh page load) */}
              {!response && (
                <button
                  onClick={() => handleSubmit(lastBirthData as BirthData)}
                  disabled={loading}
                  style={{
                    padding: "0.35rem 0.85rem",
                    border: "none",
                    borderRadius: "var(--r-sm)",
                    background: "linear-gradient(90deg, var(--c-emerald) 0%, var(--signal) 100%)",
                    color: "#070B1D",
                    fontSize: "0.72rem",
                    fontFamily: "var(--font-mono)",
                    fontWeight: 700,
                    cursor: loading ? "not-allowed" : "pointer",
                    letterSpacing: "0.06em",
                    textTransform: "uppercase",
                    opacity: loading ? 0.5 : 1,
                  }}
                >
                  {loading ? "Calculating…" : "▶ Run Analysis"}
                </button>
              )}
              <button
                onClick={() => setFormExpanded(true)}
                style={{
                  padding: "0.3rem 0.7rem",
                  border: "1px solid var(--line-mid)",
                  borderRadius: "var(--r-sm)",
                  background: "transparent",
                  color: "var(--signal)",
                  fontSize: "0.72rem",
                  fontFamily: "var(--font-mono)",
                  cursor: "pointer",
                  letterSpacing: "0.06em",
                }}
              >
                ↺ New Reading
              </button>
            </div>
          </div>
        ) : (
          <BirthDataForm onSubmit={handleSubmit} loading={loading} initialData={lastBirthData} />
        )}

        {error && <div style={s.errorBox}>{error}</div>}

        {loading && (
          <CalculationLoader
            engineLabels={selectedEngineLabels}
            modeLabel={selectedWorkflow.label}
          />
        )}

        {response && (
          <div className="result-reveal">
            {response.total_time_ms != null && (
              <div style={{ ...s.meta, marginBottom: "0.5rem" }}>
                {response.reading_id && (
                  <span>Reading: {response.reading_id}</span>
                )}
                <span>{response.total_time_ms}ms</span>
                <span>{Object.keys(outputsMap).length} engines</span>
              </div>
            )}

            <div style={s.resultShell}>
              <button
                style={{
                  ...s.synthesisButton,
                  ...(activeTab === "witness" ? s.synthesisButtonActive : {}),
                }}
                onClick={() => setActiveTab("witness")}
              >
                ◈ Synthesis
              </button>
              <EngineGrid
                engines={selectedGridEngines}
                activeEngineId={activeTab}
                availableEngineIds={availableEngineIds}
                onSelect={setActiveTab}
              />
            </div>

            {/* Active content */}
            {activeTab === "witness" && response.witness_layer && (
              <WitnessLayer data={response.witness_layer} />
            )}
            {activeTab === "witness" && !response.witness_layer && (
              <p style={{ color: "var(--text-dim)", fontSize: "0.9rem" }}>
                No witness layer returned for this reading.
              </p>
            )}

            {activeTab !== "witness" && (() => {
              const engine = engineMap.get(activeTab);
              if (!engine) {
                return (
                  <EngineCard
                    name={engineLabel(activeTab)}
                    status="idle"
                  >
                    <p style={{ color: "var(--text-dim)", fontSize: "0.9rem", lineHeight: 1.6 }}>
                      No data returned for this engine.
                    </p>
                  </EngineCard>
                );
              }
              return (
                <EngineCard
                  name={engineLabel(activeTab)}
                  status="ready"
                >
                  {renderEngine(activeTab as EngineId, engine.result)}
                </EngineCard>
              );
            })()}
          </div>
        )}

        {!response && !loading && !error && (
          <EmptyState
            label={selectedWorkflow.label.toLowerCase()}
            glyph={COMPASS_OPTIONS.find(o => o.id === selectedCompass)?.glyph ?? "◈"}
          />
        )}
      </main>
    </div>
  );
}
