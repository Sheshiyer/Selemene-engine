"use client";

import { useState, useEffect, useCallback } from "react";
import { useRouter } from "next/navigation";
import NavBar from "@/components/NavBar";
import BirthDataForm from "@/components/BirthDataForm";
import WitnessLayer from "@/components/WitnessLayer";
import EngineCard from "@/components/EngineCard";
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

function engineLabel(id: string): string {
  return id
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
    fontFamily: "'Exo 2', sans-serif",
    fontSize: "1.5rem",
    fontWeight: 800,
    color: "var(--text)",
  },
  tabBar: {
    display: "flex",
    gap: "0.25rem",
    overflowX: "auto" as const,
    padding: "0.25rem 0",
    borderBottom: "1px solid var(--line)",
  },
  tab: {
    padding: "0.5rem 0.75rem",
    fontSize: "0.8rem",
    fontWeight: 500,
    color: "var(--text-muted)",
    borderRadius: "var(--radius) var(--radius) 0 0",
    whiteSpace: "nowrap" as const,
    cursor: "pointer",
    border: "none",
    background: "transparent",
    transition: "all 0.15s",
    fontFamily: "'Space Grotesk', sans-serif",
  },
  tabActive: {
    background: "var(--gold-soft)",
    color: "var(--gold)",
    borderBottom: "2px solid var(--gold)",
  },
  tabHasData: {
    color: "var(--text)",
  },
  errorBox: {
    padding: "1rem",
    background: "rgba(239,107,115,0.1)",
    border: "1px solid var(--danger)",
    borderRadius: "var(--radius)",
    color: "var(--danger)",
    fontSize: "0.9rem",
  },
  loadingPulse: {
    color: "var(--text-muted)",
    fontSize: "0.9rem",
    fontStyle: "italic",
    textAlign: "center" as const,
    padding: "2rem 0",
  },
  meta: {
    display: "flex",
    gap: "1rem",
    fontSize: "0.8rem",
    color: "var(--text-muted)",
    fontFamily: "'IBM Plex Mono', monospace",
  },
};

/* ── Page ────────────────────────────────────────────── */

export default function EnginesPage() {
  const router = useRouter();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [response, setResponse] = useState<WorkflowResponse | null>(null);
  const [activeTab, setActiveTab] = useState<string>("witness");
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
      const res = await executeWorkflow("full-spectrum", birthData, key);
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
  }, []);

  return (
    <div style={s.page}>
      <NavBar />
      <main style={s.content}>
        <h1 style={s.heading}>Full-Spectrum Analysis</h1>
        <BirthDataForm onSubmit={handleSubmit} loading={loading} initialData={lastBirthData} />

        {error && <div style={s.errorBox}>{error}</div>}

        {loading && (
          <p style={s.loadingPulse}>
            Running all 16 engines… this may take a moment.
          </p>
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

            {/* Tab bar */}
            <div style={s.tabBar}>
              <button
                style={{
                  ...s.tab,
                  ...(activeTab === "witness" ? s.tabActive : {}),
                }}
                onClick={() => setActiveTab("witness")}
              >
                ◈ Synthesis
              </button>
              {ENGINE_IDS.map((id) => {
                const has = engineMap.has(id);
                return (
                  <button
                    key={id}
                    style={{
                      ...s.tab,
                      ...(activeTab === id ? s.tabActive : {}),
                      ...(has && activeTab !== id ? s.tabHasData : {}),
                    }}
                    onClick={() => setActiveTab(id)}
                  >
                    {engineLabel(id)}
                  </button>
                );
              })}
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
                    <p style={{ color: "var(--text-dim)", fontSize: "0.9rem" }}>
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
          <p
            style={{
              color: "var(--text-dim)",
              textAlign: "center",
              padding: "3rem 0",
              fontSize: "0.95rem",
            }}
          >
            Enter your birth data above and run the full-spectrum analysis.
          </p>
        )}
      </main>
    </div>
  );
}
