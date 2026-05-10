"use client";

import { useState, useEffect, use } from "react";
import { useRouter } from "next/navigation";
import NavBar from "@/components/NavBar";
import WitnessLayer from "@/components/WitnessLayer";
import EngineCard from "@/components/EngineCard";
import { getApiKey, isAuthenticated } from "@/lib/auth";
import { getReading, type WorkflowResponse } from "@/lib/api";

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

function engineLabel(id: string): string {
  return id.split("-").map((w) => w.charAt(0).toUpperCase() + w.slice(1)).join(" ");
}

function renderEngine(id: string, result: Record<string, unknown>) {
  switch (id) {
    case "panchanga": return <Panchanga result={result} />;
    case "human-design": return <HumanDesign result={result} />;
    case "gene-keys": return <GeneKeys result={result} />;
    case "vimshottari": return <Vimshottari result={result} />;
    case "numerology": return <Numerology result={result} />;
    case "biorhythm": return <Biorhythm result={result} />;
    case "vedic-clock": return <VedicClock result={result} />;
    case "transits": return <Transits result={result} />;
    case "biofield": return <BiofieldView result={result} />;
    case "tarot": return <Tarot result={result} />;
    case "i-ching": return <IChing result={result} />;
    case "sacred-geometry": return <SacredGeometry result={result} />;
    case "sigil-forge": return <SigilForge result={result} />;
    case "enneagram": return <Enneagram result={result} />;
    case "nadabrahman": return <Nadabrahman result={result} />;
    case "face-reading": return <FaceReading result={result} />;
    case "raaga": return <RaagaView result={result} />;
    default: return <GenericEngineView result={result} />;
  }
}

const s = {
  page: { flex: 1, display: "flex", flexDirection: "column" as const, background: "var(--bg)", minHeight: "100vh" },
  content: { maxWidth: 1100, width: "100%", margin: "0 auto", padding: "1.5rem", display: "flex", flexDirection: "column" as const, gap: "1.5rem" },
  header: { display: "flex", alignItems: "center", gap: "1rem" },
  back: {
    background: "none", border: "1px solid var(--line)", borderRadius: "var(--radius)",
    color: "var(--text-muted)", cursor: "pointer", padding: "0.35rem 0.75rem",
    fontSize: "0.8rem", fontFamily: "'Space Grotesk', sans-serif",
    flexShrink: 0 as const,
  },
  heading: { fontFamily: "'Exo 2', sans-serif", fontSize: "1.4rem", fontWeight: 800, color: "var(--text)", margin: 0 },
  meta: { display: "flex", gap: "1rem", fontSize: "0.8rem", color: "var(--text-muted)", fontFamily: "'IBM Plex Mono', monospace" },
  tabBar: { display: "flex", gap: "0.25rem", overflowX: "auto" as const, padding: "0.25rem 0", borderBottom: "1px solid var(--line)" },
  tab: {
    padding: "0.5rem 0.75rem", fontSize: "0.8rem", fontWeight: 500, color: "var(--text-muted)",
    borderRadius: "var(--radius) var(--radius) 0 0", whiteSpace: "nowrap" as const, cursor: "pointer",
    border: "none", background: "transparent", transition: "all 0.15s", fontFamily: "'Space Grotesk', sans-serif",
  },
  tabActive: { background: "var(--gold-soft)", color: "var(--gold)", borderBottom: "2px solid var(--gold)" },
  tabHasData: { color: "var(--text)" },
  loading: { color: "var(--text-muted)", textAlign: "center" as const, padding: "3rem 0", fontStyle: "italic" },
  errorBox: { padding: "1rem", background: "rgba(239,107,115,0.1)", border: "1px solid var(--danger)", borderRadius: "var(--radius)", color: "var(--danger)" },
};

export default function ReadingDetailPage({ params }: { params: Promise<{ id: string }> }) {
  const { id } = use(params);
  const router = useRouter();
  const [reading, setReading] = useState<WorkflowResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loadingData, setLoadingData] = useState(true);
  const [activeTab, setActiveTab] = useState<string>("witness");

  useEffect(() => {
    if (!isAuthenticated()) { router.replace("/auth"); return; }
    const key = getApiKey();
    if (!key) return;
    setLoadingData(true);
    getReading(id, key)
      .then((res) => { setReading(res); })
      .catch((err: unknown) => setError(err instanceof Error ? err.message : "Failed to load reading."))
      .finally(() => setLoadingData(false));
  }, [id, router]);

  const outputsMap = reading
    ? (reading.engine_outputs ?? reading.engine_results ?? {})
    : {};
  const engineIds = Object.keys(outputsMap);

  return (
    <div style={s.page}>
      <NavBar />
      <main style={s.content}>
        <div style={s.header}>
          <button style={s.back} onClick={() => router.push("/readings")}>← Readings</button>
          <div>
            <h1 style={s.heading}>{reading?.workflow_id || "Reading"}</h1>
            {reading?.timestamp && (
              <p style={{ ...s.meta, marginTop: "0.25rem" }}>
                {new Date(reading.timestamp).toLocaleString()} · {id}
                {reading.total_time_ms != null && ` · ${reading.total_time_ms}ms · ${engineIds.length} engines`}
              </p>
            )}
          </div>
        </div>

        {error && <div style={s.errorBox}>{error}</div>}
        {loadingData && <p style={s.loading}>Loading reading…</p>}

        {!loadingData && reading && (
          <>
            {/* Tab bar */}
            <div style={s.tabBar}>
              <button
                style={{ ...s.tab, ...(activeTab === "witness" ? s.tabActive : {}) }}
                onClick={() => setActiveTab("witness")}
              >
                ◈ Synthesis
              </button>
              {engineIds.map((eid) => (
                <button
                  key={eid}
                  style={{
                    ...s.tab,
                    ...(activeTab === eid ? s.tabActive : s.tabHasData),
                  }}
                  onClick={() => setActiveTab(eid)}
                >
                  {engineLabel(eid)}
                </button>
              ))}
            </div>

            {activeTab === "witness" && reading.witness_layer && (
              <WitnessLayer data={reading.witness_layer} />
            )}
            {activeTab === "witness" && !reading.witness_layer && (
              <p style={{ color: "var(--text-dim)", fontSize: "0.9rem" }}>No synthesis layer for this reading.</p>
            )}

            {activeTab !== "witness" && (() => {
              const engine = outputsMap[activeTab];
              if (!engine) return (
                <EngineCard name={engineLabel(activeTab)} status="idle">
                  <p style={{ color: "var(--text-dim)", fontSize: "0.9rem" }}>No data for this engine.</p>
                </EngineCard>
              );
              return (
                <EngineCard name={engineLabel(activeTab)} status="ready">
                  {renderEngine(activeTab, engine.result)}
                </EngineCard>
              );
            })()}
          </>
        )}

        {!loadingData && !error && !reading && (
          <p style={s.loading}>Reading not found.</p>
        )}
      </main>
    </div>
  );
}

