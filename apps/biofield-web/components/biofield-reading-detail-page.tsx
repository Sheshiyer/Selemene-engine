"use client";

import Link from "next/link";
import { useCallback, useEffect, useMemo, useState, useSyncExternalStore } from "react";
import type { BiofieldReadingDetail } from "@selemene/biofield-domain";
import { BiofieldClientError } from "@selemene/biofield-api-client";
import {
  clearStoredAuthSession,
  getStoredAuthSession,
  subscribeToAuthSession,
} from "@/lib/auth";
import { createBiofieldClient } from "@/lib/api";
import { useRouter } from "next/navigation";

function pretty(value: unknown): string {
  return JSON.stringify(value, null, 2);
}

export function BiofieldReadingDetailPage({ readingId }: { readingId: string }) {
  const router = useRouter();
  const authSession = useSyncExternalStore(
    subscribeToAuthSession,
    getStoredAuthSession,
    () => null,
  );
  const [reading, setReading] = useState<BiofieldReadingDetail | null>(null);
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

  const loadReading = useCallback(async () => {
    if (!client) {
      return;
    }

    setIsLoading(true);
    setErrorMessage(null);

    try {
      const response = await client.getReading(readingId);
      setReading(response);
    } catch (error) {
      if (error instanceof BiofieldClientError && error.status === 401) {
        clearStoredAuthSession();
        router.replace("/login");
        return;
      }
      setErrorMessage(error instanceof Error ? error.message : "Failed to load reading detail.");
    } finally {
      setIsLoading(false);
    }
  }, [client, readingId, router]);

  useEffect(() => {
    if (!authSession || !client) {
      return;
    }
    void loadReading();
  }, [authSession, client, loadReading]);

  if (!authSession) {
    return (
      <section className="biofield-panel">
        <p className="biofield-eyebrow">Reading detail</p>
        <p className="biofield-copy">Checking your biofield access…</p>
      </section>
    );
  }

  if (isLoading) {
    return (
      <section className="biofield-panel">
        <p className="biofield-eyebrow">Reading detail</p>
        <p className="biofield-copy">Loading reading {readingId}…</p>
      </section>
    );
  }

  return (
    <section className="biofield-stack">
      <section className="biofield-panel">
        <div className="biofield-shell-nav" style={{ marginBottom: 12 }}>
          <div>
            <p className="biofield-eyebrow">BF1-07 reading detail</p>
            <h2 className="biofield-title" style={{ fontSize: "2rem" }}>{readingId}</h2>
            <p className="biofield-copy">Real API-backed detail from /api/v1/biofield/readings/:id.</p>
          </div>
          <div className="biofield-actions">
            <button className="biofield-link" onClick={() => void loadReading()} type="button">Refresh</button>
            <Link className="biofield-link" href="/history">Back to history</Link>
          </div>
        </div>

        {errorMessage ? <p className="biofield-error">{errorMessage}</p> : null}

        {!reading ? (
          <p className="biofield-copy">Reading not found.</p>
        ) : (
          <div className="biofield-grid">
            <article className="biofield-panel">
              <p className="biofield-kicker">Session</p>
              <p className="biofield-copy">{reading.session_id}</p>
            </article>
            <article className="biofield-panel">
              <p className="biofield-kicker">Created</p>
              <p className="biofield-copy">{reading.created_at}</p>
            </article>
            <article className="biofield-panel">
              <p className="biofield-kicker">Artifacts</p>
              <p className="biofield-copy">{reading.artifacts.length}</p>
            </article>
          </div>
        )}
      </section>

      {reading ? (
        <section className="biofield-panel">
          <p className="biofield-eyebrow">Result payload</p>
          <pre className="biofield-copy" style={{ overflowX: "auto", whiteSpace: "pre-wrap" }}>
            {pretty(reading.result)}
          </pre>
        </section>
      ) : null}
    </section>
  );
}
