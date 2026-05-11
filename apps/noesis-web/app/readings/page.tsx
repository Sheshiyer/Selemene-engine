"use client";

import { useState, useEffect } from "react";
import { useRouter } from "next/navigation";
import NavBar from "@/components/NavBar";
import { getApiKey, isAuthenticated } from "@/lib/auth";
import { getReadings, type ReadingSummary } from "@/lib/api";

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
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!isAuthenticated()) {
      router.replace("/auth");
      return;
    }

    const key = getApiKey();
    if (!key) return;

    setLoading(true);
    getReadings(key)
      .then((res) => {
        setReadings(res.readings ?? []);
      })
      .catch((err: unknown) => {
        if (err instanceof Error) {
          setError(err.message);
        } else {
          setError("Failed to load readings.");
        }
      })
      .finally(() => setLoading(false));
  }, [router]);

  return (
    <div style={s.page}>
      <NavBar />
      <main style={s.content}>
        <h1 style={s.heading}>Readings History</h1>

        {error && <div style={s.errorBox}>{error}</div>}
        {loading && <p style={s.loading}>Loading readings…</p>}

        {!loading && readings.length === 0 && !error && (
          <p style={s.empty}>
            No readings yet. Run a full-spectrum analysis to create your first reading.
          </p>
        )}

        {readings.map((r) => (
          <div
            key={r.reading_id}
            style={s.cardClickable}
            onClick={() => router.push(`/readings/${r.reading_id}`)}
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
              {r.birth_data?.name && (
                <span style={{ fontSize: "0.8rem", color: "var(--gold)" }}>{r.birth_data.name}</span>
              )}
              {r.birth_data?.date && (
                <span style={s.readingId}>{r.birth_data.date}</span>
              )}
            </div>
            <div style={{ display: "flex", alignItems: "center", gap: "0.5rem" }}>
              <span style={s.badge}>{r.engine_count} engines</span>
              <span style={{ color: "var(--text-muted)", fontSize: "1rem" }}>→</span>
            </div>
          </div>
        ))}
      </main>
    </div>
  );
}
