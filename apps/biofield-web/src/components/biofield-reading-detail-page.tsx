"use client";

import Link from "next/link";
import { useCallback, useEffect, useMemo, useState, useSyncExternalStore } from "react";
import type {
  BiofieldBaselineSummary,
  BiofieldExportResult,
  BiofieldReadingDetail,
  BiofieldReprocessResult,
} from "@selemene/biofield-domain";
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

const numberFormatter = new Intl.NumberFormat("en-IN", {
  maximumFractionDigits: 4,
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

function formatMetricValue(value: number | null | undefined): string {
  if (typeof value !== "number" || Number.isNaN(value)) {
    return "—";
  }
  return numberFormatter.format(value);
}

function formatSignedMetricValue(value: number | null | undefined): string {
  if (typeof value !== "number" || Number.isNaN(value)) {
    return "—";
  }
  const formatted = numberFormatter.format(Math.abs(value));
  if (value > 0) {
    return `+${formatted}`;
  }
  if (value < 0) {
    return `-${formatted}`;
  }
  return formatted;
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

function downloadExportBundle(result: BiofieldExportResult) {
  const blob = new Blob([JSON.stringify(result.bundle, null, 2)], {
    type: result.mime_type,
  });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = result.file_name;
  document.body.appendChild(anchor);
  anchor.click();
  anchor.remove();
  URL.revokeObjectURL(url);
}

export function BiofieldReadingDetailPage({ readingId }: { readingId: string }) {
  const router = useRouter();
  const authSession = useSyncExternalStore(
    subscribeToAuthSession,
    getStoredAuthSession,
    () => null,
  );
  const [reading, setReading] = useState<BiofieldReadingDetail | null>(null);
  const [baselines, setBaselines] = useState<BiofieldBaselineSummary[]>([]);
  const [selectedBaselineId, setSelectedBaselineId] = useState("");
  const [reprocessResult, setReprocessResult] = useState<BiofieldReprocessResult | null>(null);
  const [latestExport, setLatestExport] = useState<BiofieldExportResult | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isReprocessing, setIsReprocessing] = useState(false);
  const [isExporting, setIsExporting] = useState(false);
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

  const selectedBaseline = useMemo(
    () => baselines.find((baseline) => baseline.baseline_id === selectedBaselineId) ?? null,
    [baselines, selectedBaselineId],
  );

  const handleAuthFailure = useCallback(() => {
    clearStoredAuthSession();
    setReading(null);
    setBaselines([]);
    router.replace("/login");
  }, [router]);

  const loadPageData = useCallback(
    async (baselineId?: string) => {
      if (!client) {
        return;
      }

      if (!readingId || readingId === "undefined") {
        router.replace("/history");
        return;
      }

      setIsLoading(true);
      setErrorMessage(null);

      try {
        const [readingResponse, baselinesResponse] = await Promise.all([
          client.getReading(readingId, { baselineId }),
          client.listBaselines(),
        ]);
        setReading(readingResponse);
        setBaselines(baselinesResponse.items);
      } catch (error) {
        if (error instanceof BiofieldClientError && error.status === 401) {
          handleAuthFailure();
          return;
        }
        setErrorMessage(getErrorMessage(error, "Failed to load biofield reading detail."));
      } finally {
        setIsLoading(false);
      }
    },
    [client, handleAuthFailure, readingId],
  );

  useEffect(() => {
    if (!authSession || !client) {
      return;
    }

    void loadPageData();
  }, [authSession, client, loadPageData]);

  async function handleRefresh() {
    await loadPageData(selectedBaselineId || undefined);
  }

  async function handleLoadComparison() {
    if (!selectedBaselineId) {
      setStatusMessage("Select a baseline to compare this reading against.");
      return;
    }

    setStatusMessage(null);
    await loadPageData(selectedBaselineId);
    setStatusMessage("Loaded baseline comparison deltas.");
  }

  async function handleClearComparison() {
    setSelectedBaselineId("");
    setStatusMessage(null);
    await loadPageData();
    setStatusMessage("Cleared baseline comparison.");
  }

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

  async function handleExport() {
    if (!client) {
      return;
    }

    setIsExporting(true);
    setErrorMessage(null);
    setStatusMessage(null);

    try {
      const response = await client.createExport({
        reading_id: readingId,
        baseline_id: selectedBaselineId || undefined,
        format: "json",
      });
      setLatestExport(response);
      downloadExportBundle(response);
      setStatusMessage(`Export ${response.file_name} generated and downloaded.`);
    } catch (error) {
      if (error instanceof BiofieldClientError && error.status === 401) {
        handleAuthFailure();
        return;
      }
      setErrorMessage(getErrorMessage(error, "Failed to create biofield export."));
    } finally {
      setIsExporting(false);
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
        <p className="biofield-copy">Verifying your session…</p>
      </section>
    );
  }

  return (
    <section className="biofield-stack">
      <section className="biofield-panel biofield-form-panel">
        <div className="biofield-toolbar">
          <div>
            <p className="biofield-eyebrow">Reading detail</p>
            <h2 className="biofield-title" style={{ fontSize: "2rem" }}>
              {analysisVersion ?? "Biofield reading detail"}
            </h2>
            <p className="biofield-copy">
              Reading <span className="biofield-mono">{readingId}</span>
            </p>
          </div>
          <div className="biofield-actions">
            <button className="biofield-link" onClick={() => void handleRefresh()} type="button">
              Refresh
            </button>
            <button className="biofield-button" disabled={isReprocessing} onClick={handleReprocess} type="button">
              {isReprocessing ? "Reprocessing…" : "Reprocess reading"}
            </button>
            <button className="biofield-button" disabled={isExporting || isLoading} onClick={handleExport} type="button">
              {isExporting ? "Exporting…" : "Export JSON bundle"}
            </button>
            <Link className="biofield-link" href="/history">
              Back to history
            </Link>
          </div>
        </div>
      </section>

      {statusMessage ? <p className="biofield-success">{statusMessage}</p> : null}
      {errorMessage ? <p className="biofield-error">{errorMessage}</p> : null}

      <section className="biofield-panel biofield-form-panel">
        <p className="biofield-eyebrow">Baseline comparison</p>
        <div className="biofield-form-inline">
          <label className="biofield-field biofield-field-grow" htmlFor="biofield-baseline-select">
            <span className="biofield-kicker">Baseline</span>
            <select
              className="biofield-input"
              id="biofield-baseline-select"
              onChange={(event) => setSelectedBaselineId(event.target.value)}
              value={selectedBaselineId}
            >
              <option value="">No baseline selected</option>
              {baselines.map((baseline) => (
                <option key={baseline.baseline_id} value={baseline.baseline_id}>
                  {baseline.name} · {baseline.reading_count} readings
                </option>
              ))}
            </select>
          </label>
          <div className="biofield-actions">
            <button className="biofield-button" disabled={isLoading || !selectedBaselineId} onClick={handleLoadComparison} type="button">
              {isLoading ? "Loading…" : "Load comparison"}
            </button>
            <button className="biofield-link" disabled={isLoading} onClick={handleClearComparison} type="button">
              Clear
            </button>
          </div>
        </div>
        {selectedBaseline ? (
          <p className="biofield-copy">
            Selected baseline <span className="biofield-mono">{selectedBaseline.baseline_id}</span>
            {selectedBaseline.notes ? ` — ${selectedBaseline.notes}` : ""}
          </p>
        ) : (
          <p className="biofield-copy">
            Select a baseline to surface deterministic deltas — coherence shifts measured against an established reference point.
          </p>
        )}
      </section>

      {latestExport ? (
        <section className="biofield-panel biofield-form-panel">
          <p className="biofield-eyebrow">Latest export</p>
          <div className="biofield-list-grid">
            <div className="biofield-list-card">
              <p className="biofield-kicker">File</p>
              <p className="biofield-copy biofield-mono">{latestExport.file_name}</p>
            </div>
            <div className="biofield-list-card">
              <p className="biofield-kicker">Bytes</p>
              <p className="biofield-metric">{latestExport.byte_size}</p>
            </div>
            <div className="biofield-list-card">
              <p className="biofield-kicker">Storage path</p>
              <p className="biofield-copy biofield-mono">{latestExport.storage_path}</p>
            </div>
          </div>
          <div className="biofield-actions">
            <button className="biofield-link" onClick={() => downloadExportBundle(latestExport)} type="button">
              Download again
            </button>
          </div>
        </section>
      ) : null}

      {reprocessResult ? (
        <section className="biofield-panel biofield-form-panel">
          <p className="biofield-eyebrow">Latest reprocess</p>
          <div className="biofield-actions">
            {reprocessResult.reading_id ? (
              <Link className="biofield-link" href={`/readings/${reprocessResult.reading_id}`}>
                Open reprocessed reading
              </Link>
            ) : null}
          </div>
        </section>
      ) : null}

      {isLoading ? (
        <section className="biofield-panel">
          <p className="biofield-eyebrow">Loading</p>
          <p className="biofield-copy">Loading reading detail and baselines…</p>
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

          {reading.comparison ? (
            <section className="biofield-panel biofield-form-panel">
              <div className="biofield-toolbar">
                <div>
                  <p className="biofield-eyebrow">Comparison deltas</p>
                  <h3 className="biofield-title" style={{ fontSize: "1.6rem" }}>
                    {reading.comparison.baseline.name}
                  </h3>
                  <p className="biofield-copy">
                    {reading.comparison.baseline.reading_count} readings · version {reading.comparison.comparison_version}
                  </p>
                </div>
              </div>
              {reading.comparison.deltas.length > 0 ? (
                <div className="biofield-list-grid">
                  {reading.comparison.deltas.map((delta) => (
                    <div className="biofield-list-card" key={delta.key}>
                      <p className="biofield-kicker">{delta.key}</p>
                      <p className="biofield-copy">Reading {formatMetricValue(delta.reading_value)}</p>
                      <p className="biofield-copy">Baseline {formatMetricValue(delta.baseline_value)}</p>
                      <p className="biofield-metric">Δ {formatSignedMetricValue(delta.absolute_delta)}</p>
                      <p className="biofield-copy">
                        Relative {delta.relative_delta == null ? "—" : formatSignedMetricValue(delta.relative_delta)}
                      </p>
                    </div>
                  ))}
                </div>
              ) : (
                <p className="biofield-copy">No comparable numeric metrics were available for this baseline.</p>
              )}
            </section>
          ) : null}

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
