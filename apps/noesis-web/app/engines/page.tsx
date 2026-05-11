"use client";

import { useState, useEffect, useCallback } from "react";
import { useRouter } from "next/navigation";
import NavBar from "@/components/NavBar";
import BirthDataForm from "@/components/BirthDataForm";
import WitnessLayer from "@/components/WitnessLayer";
import EngineCard from "@/components/EngineCard";
import CalculationLoader from "@/components/CalculationLoader";
import CompassSelector, {
  type CompassMode,
} from "@/components/CompassSelector";
import EngineGrid, { type EngineGridItem } from "@/components/EngineGrid";
import { getApiKey, isAuthenticated } from "@/lib/auth";
import {
  executeWorkflow,
  getReadings,
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
  emptyState: {
    padding: "2rem",
    border: "1px solid var(--line-mid)",
    borderRadius: "var(--r-md)",
    background:
      "radial-gradient(circle at 50% 0%, rgba(197,160,23,0.08), transparent 40%), var(--panel)",
    color: "var(--text-dim)",
    textAlign: "center" as const,
    fontSize: "0.9rem",
    lineHeight: 1.6,
  },
};

/* ── Page ────────────────────────────────────────────── */

export default function EnginesPage() {
  const router = useRouter();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [response, setResponse] = useState<WorkflowResponse | null>(null);
  const [activeTab, setActiveTab] = useState<string>("witness");
  const [selectedCompass, setSelectedCompass] =
    useState<CompassMode>("full-spectrum");
  const [lastBirthData, setLastBirthData] = useState<Partial<BirthData> | undefined>(undefined);

  useEffect(() => {
    if (!isAuthenticated()) {
      router.replace("/auth");
      return;
    }
    // Auto-fill birth data from the most recent reading
    const key = getApiKey();
    if (!key) return;
    getReadings(key)
      .then((res) => {
        const latest = res.readings?.[0];
        if (latest?.birth_data) setLastBirthData(latest.birth_data);
      })
      .catch(() => { /* silently ignore — empty form is fine */ });
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
      <NavBar />
      <main style={s.content}>
        <h1 style={s.heading}>Full-Spectrum Analysis</h1>
        <p style={s.intro}>
          Choose a compass direction before calculation. Full Spectrum keeps the
          complete 17-engine mandala; directional modes focus the workflow into
          a ritual arc.
        </p>
        <CompassSelector
          selected={selectedCompass}
          onSelect={handleCompassSelect}
        />
        <BirthDataForm onSubmit={handleSubmit} loading={loading} initialData={lastBirthData} />

        {error && <div style={s.errorBox}>{error}</div>}

        {loading && (
          <CalculationLoader
            engineLabels={selectedEngineLabels}
            modeLabel={selectedWorkflow.label}
          />
        )}

        {response && (
          <>
            {response.total_time_ms != null && (
              <div style={s.meta}>
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
          </>
        )}

        {!response && !loading && !error && (
          <p style={s.emptyState}>
            ◈ Enter your birth data above and run the{" "}
            {selectedWorkflow.label.toLowerCase()} calculation.
          </p>
        )}
      </main>
    </div>
  );
}
