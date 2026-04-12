"use client";

import Link from "next/link";
import { useCallback, useEffect, useMemo, useState, useSyncExternalStore } from "react";
import type { BiofieldReadingSummary } from "@selemene/biofield-domain";
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
  const [limit, setLimit] = useState(DEFAULT_PAGE_LIMIT);
  const [offset, setOffset] = useState(0);
  const [hasMore, setHasMore] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
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
    router.replace("/login");
  }, [router]);

  const loadReadings = useCallback(async (nextOffset: number, nextLimit: number) => {
    if (!client) {
      return;
    }

    setIsLoading(true);
    setErrorMessage(null);

    try {
      const response = await client.listReadings({ limit: nextLimit, offset: nextOffset });
      setReadings(response.items);
      setLimit(response.limit);
      setOffset(response.offset);
      setHasMore(response.items.length === response.limit);
    } catch (error) {
      if (error instanceof BiofieldClientError && error.status === 401) {
        handleAuthFailure();
        return;
      }
      setErrorMessage(getErrorMessage(error, "Failed to load biofield readings."));
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
            <p className="biofield-eyebrow">BF1-07 history</p>
            <h2 className="biofield-title" style={{ fontSize: "2rem" }}>
              Persisted readings
            </h2>
            <p className="biofield-copy">
              Every successful capture now lands as a user-scoped reading with artifact metadata and a stable detail route.
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
            <p className="biofield-kicker">Loaded items</p>
            <p className="biofield-metric">{readings.length}</p>
          </div>
          <div className="biofield-list-card">
            <p className="biofield-kicker">Page limit</p>
            <p className="biofield-metric">{limit}</p>
          </div>
          <div className="biofield-list-card">
            <p className="biofield-kicker">Offset</p>
            <p className="biofield-metric">{offset}</p>
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

      {errorMessage ? <p className="biofield-error">{errorMessage}</p> : null}

      {isLoading ? (
        <section className="biofield-panel">
          <p className="biofield-eyebrow">Loading</p>
          <p className="biofield-copy">Fetching your persisted biofield readings…</p>
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

      {!isLoading && readings.length > 0 ? (
        <section className="biofield-panel biofield-form-panel">
          <ul className="biofield-list">
            {readings.map((reading) => (
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
                    <span
                      className={`biofield-status-pill${reading.quality.sufficient_quality ? " biofield-status-pill-good" : " biofield-status-pill-warn"}`}
                    >
                      {reading.quality.sufficient_quality ? "quality ok" : "quality failed"}
                    </span>
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
            ))}
          </ul>
        </section>
      ) : null}
    </section>
  );
}
