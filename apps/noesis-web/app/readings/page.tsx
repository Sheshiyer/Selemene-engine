"use client";

import { useState, useEffect } from "react";
import { useRouter } from "next/navigation";
import NavBar from "@/components/NavBar";
import { getApiKey, isAuthenticated } from "@/lib/auth";
import { getReadings, type ReadingSummary } from "@/lib/api";
import {
  getReadingHistory,
  type CachedReading,
} from "@/lib/integrated/readingCache";

const s = {
  page: {
    flex: 1,
    display: "flex",
    flexDirection: "column" as const,
    background: "var(--bg)",
    minHeight: "100vh",
  },
  content: {
    maxWidth: 900,
    width: "100%",
    margin: "0 auto",
    padding: "1.5rem",
    display: "flex",
    flexDirection: "column" as const,
    gap: "1rem",
  },
  heading: {
    fontFamily: "var(--font-display)",
    fontSize: "1.5rem",
    fontWeight: 800,
    color: "var(--text)",
  },
  cardClickable: {
    background: "var(--bg-panel)",
    border: "1px solid var(--line)",
    borderRadius: "var(--radius)",
    padding: "1rem",
    display: "flex",
    justifyContent: "space-between",
    alignItems: "center",
    gap: "1rem",
    cursor: "pointer",
    transition: "border-color 0.15s, background 0.15s",
  },
  left: {
    display: "flex",
    flexDirection: "column" as const,
    gap: "0.25rem",
  },
  readingId: {
    fontSize: "0.8rem",
    color: "var(--gold)",
    fontFamily: "var(--font-mono)",
  },
  workflow: {
    fontSize: "0.9rem",
    color: "var(--text)",
    fontWeight: 600,
  },
  date: {
    fontSize: "0.8rem",
    color: "var(--text-muted)",
  },
  badge: {
    fontSize: "0.7rem",
    padding: "0.2rem 0.5rem",
    borderRadius: 4,
    background: "var(--emerald-soft)",
    color: "var(--emerald)",
    fontWeight: 600,
    whiteSpace: "nowrap" as const,
  },
  errorBox: {
    padding: "1rem",
    background: "rgba(239,107,115,0.1)",
    border: "1px solid var(--danger)",
    borderRadius: "var(--radius)",
    color: "var(--danger)",
    fontSize: "0.9rem",
  },
  empty: {
    color: "var(--text-dim)",
    textAlign: "center" as const,
    padding: "3rem 0",
    fontSize: "0.95rem",
  },
  loading: {
    color: "var(--text-muted)",
    textAlign: "center" as const,
    padding: "2rem 0",
    fontStyle: "italic",
  },
};

export default function ReadingsPage() {
  const router = useRouter();
  const [readings, setReadings] = useState<ReadingSummary[]>([]);
  const [localHistory, setLocalHistory] = useState<CachedReading[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [authed, setAuthed] = useState(false);

  useEffect(() => {
    // Load anonymous device history FIRST — always available, no auth needed
    setLocalHistory(getReadingHistory());
    const isAuth = isAuthenticated();
    setAuthed(isAuth);

    // If signed in, ALSO fetch the server-side reading list
    if (isAuth) {
      const key = getApiKey();
      if (!key) return;
      setLoading(true);
      getReadings(key)
        .then((res) => setReadings(res.readings ?? []))
        .catch((err: unknown) => {
          setError(err instanceof Error ? err.message : "Failed to load readings.");
        })
        .finally(() => setLoading(false));
    }
  }, [router]);

  return (
    <div style={s.page}>
      <NavBar />
      <main style={s.content}>
        <h1 style={s.heading}>Readings</h1>

        {/* ─── Recent on this device (anonymous local cache) ─────────── */}
        {localHistory.length > 0 && (
          <section style={{ display: "flex", flexDirection: "column", gap: "0.75rem" }}>
            <h2 style={{ ...s.heading, fontSize: "1rem", color: "var(--text-muted)", marginTop: "0.5rem" }}>
              ON THIS DEVICE · {localHistory.length} reading{localHistory.length === 1 ? "" : "s"}
            </h2>
            {!authed && (
              <p style={{ fontSize: "0.78rem", color: "var(--text-muted)", lineHeight: 1.5 }}>
                Anonymous — these live in your browser only.{" "}
                <a
                  href="/auth?next=/readings"
                  style={{ color: "var(--gold)", textDecoration: "underline" }}
                >
                  Save to your account
                </a>{" "}
                to access them from any device.
              </p>
            )}
            {localHistory.map((entry, i) => {
              const id = entry.payload.reading_id ?? `local-${i}`;
              const wf = (entry.payload.workflow_id as string) ?? "reading";
              const subject = entry.payload.subject?.name ?? "—";
              const birth = entry.payload.subject?.birth_date ?? "";
              return (
                <div
                  key={id}
                  style={s.cardClickable}
                  onClick={() => router.push(`/r/${id}`)}
                  onMouseEnter={(e) => {
                    (e.currentTarget as HTMLDivElement).style.borderColor = "var(--gold)";
                  }}
                  onMouseLeave={(e) => {
                    (e.currentTarget as HTMLDivElement).style.borderColor = "var(--line)";
                  }}
                >
                  <div style={s.left}>
                    <span style={s.workflow}>{wf}</span>
                    <span style={s.date}>{new Date(entry.cached_at).toLocaleString()}</span>
                    {subject !== "—" && (
                      <span style={{ fontSize: "0.8rem", color: "var(--gold)" }}>{subject}</span>
                    )}
                    {birth && <span style={s.readingId}>{birth}</span>}
                  </div>
                  <div style={{ display: "flex", alignItems: "center", gap: "0.5rem" }}>
                    {entry.claimed && <span style={s.badge}>saved</span>}
                    <span style={{ color: "var(--text-muted)", fontSize: "1rem" }}>→</span>
                  </div>
                </div>
              );
            })}
          </section>
        )}

        {/* ─── No history at all ──────────────────────────────────────── */}
        {localHistory.length === 0 && !authed && (
          <div style={{ ...s.empty, textAlign: "center" }}>
            <p style={{ marginBottom: "1rem" }}>No readings yet.</p>
            <a
              href="/get-reading"
              style={{
                display: "inline-block",
                padding: "0.7rem 1.4rem",
                background: "var(--gold)",
                color: "var(--bg)",
                borderRadius: "999px",
                fontFamily: "var(--font-mono)",
                fontSize: "0.7rem",
                letterSpacing: "0.3em",
                textTransform: "uppercase",
                textDecoration: "none",
              }}
            >
              Get a reading →
            </a>
          </div>
        )}

        {/* ─── Server-side cross-device readings (signed-in only) ────── */}
        {authed && (
          <section style={{ display: "flex", flexDirection: "column", gap: "0.75rem", marginTop: "2rem" }}>
            <h2 style={{ ...s.heading, fontSize: "1rem", color: "var(--text-muted)" }}>
              ALL READINGS · CROSS-DEVICE
            </h2>
            {error && <div style={s.errorBox}>{error}</div>}
            {loading && <p style={s.loading}>Loading readings…</p>}
            {!loading && readings.length === 0 && !error && (
              <p style={s.empty}>
                No saved readings yet. Generate one and click <em>Save this reading</em> to add it here.
              </p>
            )}
            {readings.map((r) => (
          <div
            key={r.id}
            style={s.cardClickable}
            onClick={() => router.push(`/readings/${r.id}`)}
            onMouseEnter={(e) => {
              (e.currentTarget as HTMLDivElement).style.borderColor = "var(--gold)";
              (e.currentTarget as HTMLDivElement).style.background = "var(--bg-hover, var(--bg-panel))";
            }}
            onMouseLeave={(e) => {
              (e.currentTarget as HTMLDivElement).style.borderColor = "var(--line)";
              (e.currentTarget as HTMLDivElement).style.background = "var(--bg-panel)";
            }}
          >
            <div style={s.left}>
              <span style={s.workflow}>{r.workflow_id || "full-spectrum"}</span>
              <span style={s.date}>
                {r.created_at ? new Date(r.created_at).toLocaleString() : "—"}
              </span>
              {r.input_data && 'name' in r.input_data && (r.input_data as {name?: string}).name && (
                <span style={{ fontSize: "0.8rem", color: "var(--gold)" }}>{(r.input_data as {name?: string}).name}</span>
              )}
              {r.input_data && 'date' in r.input_data && (r.input_data as {date?: string}).date && (
                <span style={s.readingId}>{(r.input_data as {date?: string}).date}</span>
              )}
            </div>
            <div style={{ display: "flex", alignItems: "center", gap: "0.5rem" }}>
              {(() => {
                const count = Object.keys(
                  r.result_data?.engine_outputs ?? r.result_data?.engine_results ?? {}
                ).length;
                return count > 0 ? <span style={s.badge}>{count} engines</span> : null;
              })()}
              <span style={{ color: "var(--text-muted)", fontSize: "1rem" }}>→</span>
            </div>
          </div>
        ))}
          </section>
        )}
      </main>
    </div>
  );
}
