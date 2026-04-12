"use client";

import Link from "next/link";
import { useCallback, useEffect, useMemo, useState, useSyncExternalStore } from "react";
import type { BiofieldReadingDetail, BiofieldReprocessResult } from "@selemene/biofield-domain";
import { BiofieldClientError } from "@selemene/biofield-api-client";
import {
  clearStoredAuthSession,
  getStoredAuthSession,
  subscribeToAuthSession,
} from "@/lib/auth";
import { createBiofieldClient } from "@/lib/api";
import { useRouter } from "next/navigation";

const METRIC_KEYS = [
  "light_quanta_density",
  "normalized_area",
  "average_intensity",
  "fractal_dimension",
  "body_symmetry",
  "pattern_regularity",
] as const;

const detailFormatter = new Intl.DateTimeFormat("en-IN", {
  dateStyle: "medium",
  timeStyle: "short",
});

function isRecord(value: unknown): value is Record<string, unknown> {
  return !!value && typeof value === "object" && !Array.isArray(value);
}

function formatTimestamp(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.valueOf())) {
    return value;
  }
  return detailFormatter.format(date);
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

function prettyJson(value: unknown): string {
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
  const [reprocessResult, setReprocessResult] = useState<BiofieldReprocessResult | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isReprocessing, setIsReprocessing] = useState(false);
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
    setReading(null);
    router.replace("/login");
  }, [router]);

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
        handleAuthFailure();
        return;
      }
      setErrorMessage(getErrorMessage(error, "Failed to load biofield reading detail."));
    } finally {
      setIsLoading(false);
    }
  }, [client, handleAuthFailure, readingId]);

  useEffect(() => {
    if (!authSession || !client) {
      return;
    }

    void loadReading();
  }, [authSession, client, loadReading]);

  async function handleReprocess() {
    if (!client) {
      return;
    }

    setIsReprocessing(true);
    setErrorMessage(null);
    setStatusMessage(null);

    try {
      const response = await client.reprocessReading(readingId);
      setReprocessResult(response);
      setStatusMessage(`Reprocessed into reading ${response.reading_id}.`);
    } catch (error) {
      if (error instanceof BiofieldClientError && error.status === 401) {
        handleAuthFailure();
        return;
      }
      setErrorMessage(getErrorMessage(error, "Failed to reprocess biofield reading."));
    } finally {
      setIsReprocessing(false);
    }
  }

  const metrics = useMemo(() => {
    if (!reading || !isRecord(reading.result)) {
      return [] as Array<{ key: string; value: unknown }>;
    }

    const metricsValue = reading.result.metrics;
    if (!isRecord(metricsValue)) {
      return [] as Array<{ key: string; value: unknown }>;
    }

    return METRIC_KEYS.filter((key) => key in metricsValue).map((key) => ({
      key,
      value: metricsValue[key],
    }));
  }, [reading]);

  const analysisVersion = useMemo(() => {
    if (!reading || !isRecord(reading.result)) {
      return null;
    }
    return typeof reading.result.analysis_version === "string"
      ? reading.result.analysis_version
      : null;
  }, [reading]);

  if (!authSession) {
    return (
      <section className="biofield-panel">
        <p className="biofield-eyebrow">Reading detail</p>
        <p className="biofield-copy">Checking your biofield access…</p>
      </section>
    );
  }

  return (
    <section className="biofield-stack">
      <section className="biofield-panel biofield-form-panel">
        <div className="biofield-toolbar">
          <div>
            <p className="biofield-eyebrow">BF2 detail and reprocess</p>
            <h2 className="biofield-title" style={{ fontSize: "2rem" }}>
              {analysisVersion ?? "Biofield reading detail"}
            </h2>
            <p className="biofield-copy">
              Reading <span className="biofield-mono">{readingId}</span>
            </p>
          </div>
          <div className="biofield-actions">
            <button className="biofield-link" onClick={() => void loadReading()} type="button">
              Refresh
            </button>
            <button className="biofield-button" disabled={isReprocessing} onClick={handleReprocess} type="button">
              {isReprocessing ? "Reprocessing…" : "Reprocess reading"}
            </button>
            <Link className="biofield-link" href="/history">
              Back to history
            </Link>
          </div>
        </div>
      </section>

      {statusMessage ? <p className="biofield-success">{statusMessage}</p> : null}
      {errorMessage ? <p className="biofield-error">{errorMessage}</p> : null}

      {reprocessResult ? (
        <section className="biofield-panel biofield-form-panel">
          <p className="biofield-eyebrow">Latest reprocess</p>
          <div className="biofield-actions">
            <Link className="biofield-link" href={`/readings/${reprocessResult.reading_id}`}>
              Open reprocessed reading
            </Link>
          </div>
        </section>
      ) : null}

      {isLoading ? (
        <section className="biofield-panel">
          <p className="biofield-eyebrow">Loading</p>
          <p className="biofield-copy">Fetching persisted reading detail…</p>
        </section>
      ) : null}

      {reading ? (
        <>
          <section className="biofield-grid">
            <article className="biofield-panel">
              <p className="biofield-kicker">Created at</p>
              <p className="biofield-metric">{formatTimestamp(reading.created_at)}</p>
            </article>
            <article className="biofield-panel">
              <p className="biofield-kicker">Session</p>
              <p className="biofield-copy biofield-mono">{reading.session_id}</p>
            </article>
            <article className="biofield-panel">
              <p className="biofield-kicker">Artifacts</p>
              <p className="biofield-metric">{reading.artifacts.length}</p>
            </article>
          </section>

          {metrics.length > 0 ? (
            <section className="biofield-panel biofield-form-panel">
              <p className="biofield-eyebrow">Metrics</p>
              <div className="biofield-list-grid">
                {metrics.map((metric) => (
                  <div className="biofield-list-card" key={metric.key}>
                    <p className="biofield-kicker">{metric.key}</p>
                    <p className="biofield-metric">{String(metric.value)}</p>
                  </div>
                ))}
              </div>
            </section>
          ) : null}

          <section className="biofield-panel biofield-form-panel">
            <p className="biofield-eyebrow">Artifacts</p>
            <div className="biofield-list-grid">
              {reading.artifacts.map((artifact, index) => (
                <div className="biofield-list-card" key={artifact.id ?? `${artifact.kind}-${index}`}>
                  <p className="biofield-kicker">{artifact.kind}</p>
                  <p className="biofield-copy">{artifact.mime_type}</p>
                  <p className="biofield-copy biofield-mono">
                    {artifact.storage_path ?? "No storage path recorded"}
                  </p>
                  <p className="biofield-copy">
                    {artifact.byte_size !== undefined ? `${artifact.byte_size} bytes` : "Byte size unavailable"}
                  </p>
                </div>
              ))}
            </div>
          </section>

          <section className="biofield-grid biofield-detail-grid">
            <article className="biofield-panel biofield-form-panel">
              <p className="biofield-eyebrow">Input</p>
              <pre className="biofield-json">{prettyJson(reading.input)}</pre>
            </article>
            <article className="biofield-panel biofield-form-panel">
              <p className="biofield-eyebrow">Quality</p>
              <pre className="biofield-json">{prettyJson(reading.quality)}</pre>
            </article>
            <article className="biofield-panel biofield-form-panel">
              <p className="biofield-eyebrow">Result</p>
              <pre className="biofield-json">{prettyJson(reading.result)}</pre>
            </article>
          </section>
        </>
      ) : null}
    </section>
  );
}
