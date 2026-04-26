"use client";

import Link from "next/link";
import { FormEvent, useCallback, useEffect, useMemo, useState, useSyncExternalStore } from "react";
import type { BiofieldBaselineSummary, BiofieldReadingSummary } from "@selemene/biofield-domain";
import { BiofieldClientError } from "@selemene/biofield-api-client";
import {
  clearStoredAuthSession,
  getStoredAuthSession,
  subscribeToAuthSession,
} from "@/lib/auth";
import { createBiofieldClient } from "@/lib/api";
import { useRouter } from "next/navigation";

const pageFormatter = new Intl.DateTimeFormat("en-IN", {
  dateStyle: "medium",
  timeStyle: "short",
});
const DEFAULT_PAGE_LIMIT = 20;

function formatTimestamp(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.valueOf())) {
    return value;
  }
  return pageFormatter.format(date);
}

function getErrorMessage(error: unknown, fallback: string): string {
  if (error instanceof BiofieldClientError) {
    if (error.details && typeof error.details === "object") {
      const detailRecord = error.details as Record<string, unknown>;
      if (typeof detailRecord.message === "string") {
        return detailRecord.message;
      }
      if (typeof detailRecord.error_message === "string") {
        return detailRecord.error_message;
      }
    }
    return error.message;
  }

  return error instanceof Error ? error.message : fallback;
}

export function BiofieldHistoryPage() {
  const router = useRouter();
  const authSession = useSyncExternalStore(
    subscribeToAuthSession,
    getStoredAuthSession,
    () => null,
  );
  const [readings, setReadings] = useState<BiofieldReadingSummary[]>([]);
  const [baselines, setBaselines] = useState<BiofieldBaselineSummary[]>([]);
  const [selectedReadingIds, setSelectedReadingIds] = useState<string[]>([]);
  const [baselineName, setBaselineName] = useState("");
  const [baselineNotes, setBaselineNotes] = useState("");
  const [limit, setLimit] = useState(DEFAULT_PAGE_LIMIT);
  const [offset, setOffset] = useState(0);
  const [hasMore, setHasMore] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [isCreatingBaseline, setIsCreatingBaseline] = useState(false);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  useEffect(() => {
    if (!authSession) {
      router.replace("/login");
    }
  }, [authSession, router]);

  const client = useMemo(() => {
    if (!authSession) {
      return null;
    }
    return createBiofieldClient(authSession.token);
  }, [authSession]);

  const handleAuthFailure = useCallback(() => {
    clearStoredAuthSession();
    setReadings([]);
    setBaselines([]);
    router.replace("/login");
  }, [router]);

  const loadReadings = useCallback(async (nextOffset: number, nextLimit: number) => {
    if (!client) {
      return;
    }

    setIsLoading(true);
    setErrorMessage(null);

    try {
      const [readingsResponse, baselinesResponse] = await Promise.all([
        client.listReadings({ limit: nextLimit, offset: nextOffset }),
        client.listBaselines(),
      ]);

      setReadings(readingsResponse.items);
      setLimit(readingsResponse.limit);
      setOffset(readingsResponse.offset);
      setHasMore(readingsResponse.items.length === readingsResponse.limit);
      setBaselines(baselinesResponse.items);
    } catch (error) {
      if (error instanceof BiofieldClientError && error.status === 401) {
        handleAuthFailure();
        return;
      }
      setErrorMessage(getErrorMessage(error, "Failed to load biofield history."));
    } finally {
      setIsLoading(false);
    }
  }, [client, handleAuthFailure]);

  useEffect(() => {
    if (!authSession || !client) {
      return;
    }

    void loadReadings(0, DEFAULT_PAGE_LIMIT);
  }, [authSession, client, loadReadings]);

  function toggleReadingSelection(readingId: string) {
    setSelectedReadingIds((current) =>
      current.includes(readingId)
        ? current.filter((id) => id !== readingId)
        : [...current, readingId],
    );
  }

  async function handleCreateBaseline(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!client || selectedReadingIds.length === 0) {
      return;
    }

    setIsCreatingBaseline(true);
    setErrorMessage(null);
    setStatusMessage(null);

    try {
      const baseline = await client.createBaseline({
        name: baselineName,
        notes: baselineNotes.trim() || undefined,
        reading_ids: selectedReadingIds,
      });
      setStatusMessage(`Baseline ${baseline.name} created from ${baseline.reading_count} readings.`);
      setBaselineName("");
      setBaselineNotes("");
      setSelectedReadingIds([]);
      await loadReadings(offset, limit);
    } catch (error) {
      if (error instanceof BiofieldClientError && error.status === 401) {
        handleAuthFailure();
        return;
      }
      setErrorMessage(getErrorMessage(error, "Failed to create biofield baseline."));
    } finally {
      setIsCreatingBaseline(false);
    }
  }

  function handlePreviousPage() {
    const nextOffset = Math.max(0, offset - limit);
    void loadReadings(nextOffset, limit);
  }

  function handleNextPage() {
    if (!hasMore) {
      return;
    }
    void loadReadings(offset + limit, limit);
  }

  if (!authSession) {
    return (
      <section className="biofield-panel">
        <p className="biofield-eyebrow">History</p>
        <p className="biofield-copy">Checking your biofield access…</p>
      </section>
    );
  }

  return (
    <section className="biofield-stack">
      <section className="biofield-panel biofield-form-panel">
        <div className="biofield-toolbar">
          <div>
            <p className="biofield-eyebrow">BF2 history and baselines</p>
            <h2 className="biofield-title" style={{ fontSize: "2rem" }}>
              Persisted readings
            </h2>
            <p className="biofield-copy">
              Create lightweight baselines from selected readings while keeping the Phase 1 history surface intact.
            </p>
          </div>
          <div className="biofield-actions">
            <button className="biofield-link" onClick={() => void loadReadings(offset, limit)} type="button">
              Refresh
            </button>
            <Link className="biofield-link" href="/viewer">
              Open viewer
            </Link>
          </div>
        </div>

        <div className="biofield-meta-grid">
          <div className="biofield-list-card">
            <p className="biofield-kicker">Loaded readings</p>
            <p className="biofield-metric">{readings.length}</p>
          </div>
          <div className="biofield-list-card">
            <p className="biofield-kicker">Baselines</p>
            <p className="biofield-metric">{baselines.length}</p>
          </div>
          <div className="biofield-list-card">
            <p className="biofield-kicker">Selected readings</p>
            <p className="biofield-metric">{selectedReadingIds.length}</p>
          </div>
          <div className="biofield-list-card">
            <p className="biofield-kicker">Page window</p>
            <p className="biofield-metric">{offset} -&gt; {offset + readings.length}</p>
          </div>
          <div className="biofield-list-card">
            <p className="biofield-kicker">Page</p>
            <p className="biofield-metric">{limit > 0 ? Math.floor(offset / limit) + 1 : 1}</p>
          </div>
        </div>
        <div className="biofield-actions">
          <button
            className="biofield-link"
            disabled={isLoading || offset === 0}
            onClick={handlePreviousPage}
            type="button"
          >
            Previous
          </button>
          <button
            className="biofield-link"
            disabled={isLoading || !hasMore}
            onClick={handleNextPage}
            type="button"
          >
            Next
          </button>
        </div>
      </section>

      {statusMessage ? <p className="biofield-success">{statusMessage}</p> : null}
      {errorMessage ? <p className="biofield-error">{errorMessage}</p> : null}

      {!isLoading && readings.length > 0 ? (
        <section className="biofield-panel biofield-form-panel">
          <p className="biofield-eyebrow">Create baseline</p>
          <form className="biofield-form" onSubmit={handleCreateBaseline}>
            <label className="biofield-field" htmlFor="biofield-baseline-name">
              <span className="biofield-kicker">Baseline name</span>
              <input
                className="biofield-input"
                id="biofield-baseline-name"
                onChange={(event) => setBaselineName(event.target.value)}
                placeholder="Morning baseline"
                required
                value={baselineName}
              />
            </label>
            <label className="biofield-field" htmlFor="biofield-baseline-notes">
              <span className="biofield-kicker">Notes</span>
              <textarea
                className="biofield-input biofield-textarea"
                id="biofield-baseline-notes"
                onChange={(event) => setBaselineNotes(event.target.value)}
                placeholder="Optional reflection about why these readings belong together"
                value={baselineNotes}
              />
            </label>
            <div className="biofield-actions">
              <button
                className="biofield-button"
                disabled={isCreatingBaseline || selectedReadingIds.length === 0 || baselineName.trim().length === 0}
                type="submit"
              >
                {isCreatingBaseline ? "Creating baseline…" : "Create baseline from selection"}
              </button>
            </div>
          </form>
        </section>
      ) : null}

      {isLoading ? (
        <section className="biofield-panel">
          <p className="biofield-eyebrow">Loading</p>
          <p className="biofield-copy">Fetching your persisted biofield readings and baselines…</p>
        </section>
      ) : null}

      {!isLoading && !errorMessage && readings.length === 0 ? (
        <section className="biofield-panel">
          <p className="biofield-eyebrow">No readings yet</p>
          <h3 className="biofield-title" style={{ fontSize: "1.6rem" }}>
            Your history is empty
          </h3>
          <p className="biofield-copy">
            Start a session in the viewer and upload a capture to create the first persisted biofield reading.
          </p>
          <div className="biofield-actions">
            <Link className="biofield-button" href="/viewer">
              Go to viewer
            </Link>
          </div>
        </section>
      ) : null}

      {!isLoading && baselines.length > 0 ? (
        <section className="biofield-panel biofield-form-panel">
          <p className="biofield-eyebrow">Existing baselines</p>
          <div className="biofield-list-grid">
            {baselines.map((baseline) => (
              <div className="biofield-list-card" key={baseline.baseline_id}>
                <p className="biofield-kicker">{baseline.name}</p>
                <p className="biofield-copy">{baseline.notes ?? "No notes"}</p>
                <p className="biofield-copy">{baseline.reading_count} readings</p>
                <p className="biofield-copy biofield-mono">{baseline.baseline_id}</p>
              </div>
            ))}
          </div>
        </section>
      ) : null}

      {!isLoading && readings.length > 0 ? (
        <section className="biofield-panel biofield-form-panel">
          <ul className="biofield-list">
            {readings.map((reading) => {
              const selected = selectedReadingIds.includes(reading.reading_id);
              return (
                <li key={reading.reading_id}>
                  <div className="biofield-toolbar">
                    <div>
                      <p className="biofield-kicker">{formatTimestamp(reading.created_at)}</p>
                      <p className="biofield-metric">
                        {reading.quality.sufficient_quality ? "Accepted capture" : "Rejected capture"}
                      </p>
                      <p className="biofield-copy">
                        Reading <span className="biofield-mono">{reading.reading_id}</span>
                      </p>
                    </div>
                    <div className="biofield-actions">
                      <label className="biofield-checkbox-row">
                        <input
                          checked={selected}
                          className="biofield-checkbox"
                          onChange={() => toggleReadingSelection(reading.reading_id)}
                          type="checkbox"
                        />
                        <span>{selected ? "Selected" : "Select for baseline"}</span>
                      </label>
                      <Link className="biofield-link" href={`/readings/${reading.reading_id}`}>
                        Open detail
                      </Link>
                    </div>
                  </div>

                  <div className="biofield-meta-grid">
                    <div className="biofield-list-card">
                      <p className="biofield-kicker">Session</p>
                      <p className="biofield-copy biofield-mono">{reading.session_id}</p>
                    </div>
                    <div className="biofield-list-card">
                      <p className="biofield-kicker">Artifact</p>
                      <p className="biofield-copy">{reading.artifact.mime_type}</p>
                    </div>
                    <div className="biofield-list-card">
                      <p className="biofield-kicker">Storage path</p>
                      <p className="biofield-copy biofield-mono">
                        {reading.artifact.storage_path ?? "Not yet linked"}
                      </p>
                    </div>
                  </div>
                </li>
              );
            })}
          </ul>
        </section>
      ) : null}
    </section>
  );
}
