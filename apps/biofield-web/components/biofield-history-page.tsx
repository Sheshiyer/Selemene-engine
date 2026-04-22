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

const formatter = new Intl.DateTimeFormat("en-IN", {
  dateStyle: "medium",
  timeStyle: "short",
});

function formatTimestamp(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.valueOf())) {
    return value;
  }
  return formatter.format(date);
}

export function BiofieldHistoryPage() {
  const router = useRouter();
  const authSession = useSyncExternalStore(
    subscribeToAuthSession,
    getStoredAuthSession,
    () => null,
  );
  const [readings, setReadings] = useState<BiofieldReadingSummary[]>([]);
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

  const loadReadings = useCallback(async () => {
    if (!client) {
      return;
    }

    setIsLoading(true);
    setErrorMessage(null);

    try {
      const response = await client.listReadings({ limit: 50, offset: 0 });
      const items = Array.isArray(response)
        ? response
        : Array.isArray((response as { items?: BiofieldReadingSummary[] }).items)
          ? (response as { items: BiofieldReadingSummary[] }).items
          : [];
      setReadings(items);
    } catch (error) {
      if (error instanceof BiofieldClientError && error.status === 401) {
        clearStoredAuthSession();
        router.replace("/login");
        return;
      }
      setErrorMessage(error instanceof Error ? error.message : "Failed to load biofield history.");
    } finally {
      setIsLoading(false);
    }
  }, [client, router]);

  useEffect(() => {
    if (!authSession || !client) {
      return;
    }
    void loadReadings();
  }, [authSession, client, loadReadings]);

  if (!authSession) {
    return (
      <section className="biofield-panel">
        <p className="biofield-eyebrow">History</p>
        <p className="biofield-copy">Checking your biofield access…</p>
      </section>
    );
  }

  if (isLoading) {
    return (
      <section className="biofield-panel">
        <p className="biofield-eyebrow">History</p>
        <p className="biofield-copy">Loading persisted readings…</p>
      </section>
    );
  }

  return (
    <section className="biofield-panel">
      <div className="biofield-shell-nav" style={{ marginBottom: 12 }}>
        <div>
          <p className="biofield-eyebrow">BF1-07 history</p>
          <h2 className="biofield-title" style={{ fontSize: "2rem" }}>Persisted readings</h2>
          <p className="biofield-copy">Real API-backed list from /api/v1/biofield/readings.</p>
        </div>
        <div className="biofield-actions">
          <button className="biofield-link" onClick={() => void loadReadings()} type="button">Refresh</button>
          <Link className="biofield-link" href="/viewer">Open viewer</Link>
        </div>
      </div>

      {errorMessage ? <p className="biofield-error">{errorMessage}</p> : null}

      {readings.length === 0 ? (
        <p className="biofield-copy">No persisted readings yet. Capture from viewer to populate history.</p>
      ) : (
        <ul className="biofield-list">
          {readings.map((reading) => (
            <li key={reading.reading_id}>
              <p className="biofield-kicker">{reading.reading_id}</p>
              <p className="biofield-metric">{formatTimestamp(reading.created_at)}</p>
              <p className="biofield-copy">Session {reading.session_id}</p>
              <div className="biofield-actions">
                <Link className="biofield-link" href={`/readings/${reading.reading_id}`}>Open detail</Link>
              </div>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
