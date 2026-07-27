"use client";

import { useCallback, useEffect, useState } from "react";
import { usePathname, useRouter, useSearchParams } from "next/navigation";
import { ActionRail } from "@/components/admin-primitives";
import { StateBanner, StatePanel, TableEmptyStateRow } from "@/components/admin-state";
import { DrawerSurface } from "@/components/overlay-surface";
import { PageShell } from "@/components/page-shell";
import { getAuthToken } from "@/lib/auth";
import {
  ApiClientError,
  getAdminLivingReading,
  getAdminLivingReadings
} from "@/lib/api";
import { buildQueryString, getNumberParam, getStringParam } from "@/lib/url-query";
import type {
  AdminLivingReadingDetail,
  AdminLivingReadingItem
} from "@/types/admin";

const EDITORIAL_STATES = [
  "imported",
  "needs_review",
  "in_review",
  "approved",
  "rejected",
  "published",
  "archived"
];

function formatDateTime(value: string | null): string {
  if (!value) return "--";
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? "--" : date.toLocaleString();
}

function formatBytes(value: number | null): string {
  if (value === null) return "--";
  if (value < 1024) return `${value} B`;
  if (value < 1024 * 1024) return `${(value / 1024).toFixed(1)} KB`;
  return `${(value / (1024 * 1024)).toFixed(1)} MB`;
}

function shortChecksum(value: string | null): string {
  return value ? `${value.slice(0, 12)}\u2026${value.slice(-8)}` : "--";
}

function subjectLabel(item: AdminLivingReadingItem): string {
  return item.subjects.length
    ? item.subjects.map((subject) => `${subject.canonical_name} (${subject.role})`).join(", ")
    : "Unlinked";
}

export default function LivingReadingsPage() {
  const router = useRouter();
  const pathname = usePathname();
  const searchParams = useSearchParams();

  const ownerUserId = getStringParam(searchParams, "owner_user_id");
  const subjectId = getStringParam(searchParams, "subject_id");
  const relationshipId = getStringParam(searchParams, "relationship_id");
  const sourceId = getStringParam(searchParams, "source_id");
  const importRunId = getStringParam(searchParams, "import_run_id");
  const editorialState = getStringParam(searchParams, "editorial_state");
  const limit = getNumberParam(searchParams, "limit", 25, 1, 100);
  const offset = getNumberParam(searchParams, "offset", 0, 0, 10000);

  const [loading, setLoading] = useState(true);
  const [detailLoading, setDetailLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [items, setItems] = useState<AdminLivingReadingItem[]>([]);
  const [total, setTotal] = useState(0);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [detail, setDetail] = useState<AdminLivingReadingDetail | null>(null);

  const updateQuery = useCallback(
    (updates: Record<string, string | number | undefined>) => {
      const nextQuery = buildQueryString(searchParams, updates);
      router.replace(nextQuery ? `${pathname}?${nextQuery}` : pathname, {
        scroll: false
      });
    },
    [pathname, router, searchParams]
  );

  const loadData = useCallback(async () => {
    return getAdminLivingReadings(getAuthToken() ?? undefined, {
      owner_user_id: getStringParam(searchParams, "owner_user_id") || undefined,
      subject_id: getStringParam(searchParams, "subject_id") || undefined,
      relationship_id: getStringParam(searchParams, "relationship_id") || undefined,
      source_id: getStringParam(searchParams, "source_id") || undefined,
      import_run_id: getStringParam(searchParams, "import_run_id") || undefined,
      editorial_state: getStringParam(searchParams, "editorial_state") || undefined,
      limit: getNumberParam(searchParams, "limit", 25, 1, 100),
      offset: getNumberParam(searchParams, "offset", 0, 0, 10000)
    });
  }, [searchParams]);

  useEffect(() => {
    let cancelled = false;
    async function run() {
      setLoading(true);
      setError(null);
      try {
        const result = await loadData();
        if (!cancelled) {
          setItems(result.items);
          setTotal(result.total);
        }
      } catch (err) {
        if (!cancelled) {
          setError(
            err instanceof ApiClientError
              ? err.payload?.error || err.message
              : "Failed to load living readings"
          );
        }
      } finally {
        if (!cancelled) setLoading(false);
      }
    }
    void run();
    return () => {
      cancelled = true;
    };
  }, [loadData]);

  const openDetail = useCallback(async (readingId: string) => {
    setSelectedId(readingId);
    setDetail(null);
    setDetailLoading(true);
    setError(null);
    try {
      setDetail(await getAdminLivingReading(getAuthToken() ?? undefined, readingId));
    } catch (err) {
      setError(
        err instanceof ApiClientError
          ? err.payload?.error || err.message
          : "Failed to load living reading detail"
      );
      setSelectedId(null);
    } finally {
      setDetailLoading(false);
    }
  }, []);

  const closeDetail = useCallback(() => {
    setSelectedId(null);
    setDetail(null);
  }, []);

  const hasPrev = offset > 0;
  const hasNext = offset + items.length < total;

  return (
    <PageShell
      eyebrow="Living Archive"
      title="Living Readings Archive"
      summary="Canonical historical readings with explicit owner, subject, relationship, source, evidence, and editorial lineage."
      actions={
        <ActionRail label="Living archive actions">
          <button type="button" onClick={() => window.location.reload()}>
            Refresh archive
          </button>
        </ActionRail>
      }
    >
      <StateBanner
        variant="success"
        title="Metadata-only artifact review"
        description="Artifact locators and checksums are shown for provenance. This browser never fetches artifact bytes."
      />

      <div className="panel-inline">
        <label>
          Owner UUID
          <input
            value={ownerUserId}
            onChange={(event) =>
              updateQuery({ owner_user_id: event.target.value || undefined, offset: undefined })
            }
            placeholder="owner user UUID"
          />
        </label>
        <label>
          Subject UUID
          <input
            value={subjectId}
            onChange={(event) =>
              updateQuery({ subject_id: event.target.value || undefined, offset: undefined })
            }
            placeholder="corpus subject UUID"
          />
        </label>
        <label>
          Relationship UUID
          <input
            value={relationshipId}
            onChange={(event) =>
              updateQuery({ relationship_id: event.target.value || undefined, offset: undefined })
            }
            placeholder="relationship UUID"
          />
        </label>
        <label>
          Source UUID
          <input
            value={sourceId}
            onChange={(event) =>
              updateQuery({ source_id: event.target.value || undefined, offset: undefined })
            }
            placeholder="source UUID"
          />
        </label>
        <label>
          Import run UUID
          <input
            value={importRunId}
            onChange={(event) =>
              updateQuery({ import_run_id: event.target.value || undefined, offset: undefined })
            }
            placeholder="import run UUID"
          />
        </label>
        <label>
          Editorial state
          <select
            value={editorialState}
            onChange={(event) =>
              updateQuery({ editorial_state: event.target.value || undefined, offset: undefined })
            }
          >
            <option value="">All states</option>
            {EDITORIAL_STATES.map((state) => (
              <option key={state} value={state}>
                {state.replaceAll("_", " ")}
              </option>
            ))}
          </select>
        </label>
      </div>

      {error ? <StateBanner variant="error" title={error} /> : null}

      {loading ? (
        <StatePanel
          variant="loading"
          title="Loading living archive"
          description="Resolving bounded archive rows and their current provenance."
        />
      ) : (
        <article className="panel">
          <h3>Archived readings</h3>
          <div className="table-wrap compact">
            <table>
              <thead>
                <tr>
                  <th>Reading</th>
                  <th>Owner</th>
                  <th>Subjects</th>
                  <th>Relationship</th>
                  <th>Editorial</th>
                  <th>Source checksum</th>
                  <th>Captured</th>
                </tr>
              </thead>
              <tbody>
                {items.map((item) => (
                  <tr key={item.id}>
                    <td>
                      <button
                        type="button"
                        className="link-btn table-primary"
                        onClick={() => void openDetail(item.id)}
                      >
                        {item.title}
                      </button>
                      <div className="helper">
                        {item.reading_type} · {item.producer_kind}
                      </div>
                    </td>
                    <td>
                      {item.owner_email}
                      <div className="helper">{item.owner_user_id}</div>
                    </td>
                    <td className="cell-wrap">{subjectLabel(item)}</td>
                    <td>{item.relationship_label ?? item.relationship_kind ?? "--"}</td>
                    <td>
                      <span className="pill">{item.editorial_state ?? "unreviewed"}</span>
                      <div className="helper">{item.editorial_visibility ?? "--"}</div>
                    </td>
                    <td title={item.source_sha256 ?? undefined}>
                      <code>{shortChecksum(item.source_sha256)}</code>
                    </td>
                    <td>{formatDateTime(item.captured_at ?? item.created_at)}</td>
                  </tr>
                ))}
                {items.length === 0 ? (
                  <TableEmptyStateRow
                    colSpan={7}
                    title="No living readings found"
                    description="Adjust owner, subject, relationship, source, run, or editorial filters."
                  />
                ) : null}
              </tbody>
            </table>
          </div>

          <div className="table-pagination">
            <button
              type="button"
              disabled={!hasPrev}
              onClick={() => updateQuery({ offset: Math.max(0, offset - limit) })}
            >
              Previous
            </button>
            <span className="helper">
              {total === 0 ? 0 : offset + 1}&ndash;{Math.min(offset + items.length, total)} of {total}
            </span>
            <button
              type="button"
              disabled={!hasNext}
              onClick={() => updateQuery({ offset: offset + limit })}
            >
              Next
            </button>
          </div>
        </article>
      )}

      <DrawerSurface
        open={Boolean(selectedId)}
        onClose={closeDetail}
        eyebrow="Living Reading"
        title={detail?.reading.title ?? "Archive detail"}
        summary={detail ? `${detail.reading.owner_email} · ${detail.reading.reading_type}` : undefined}
        footer={
          <button type="button" onClick={closeDetail}>
            Close
          </button>
        }
      >
        {detailLoading ? (
          <StatePanel
            variant="loading"
            title="Loading archive detail"
            description="Resolving provenance, artifacts, evidence, and editorial history."
          />
        ) : null}
        {detail ? (
          <div className="grid overlay-detail-grid">
            <article className="detail-card detail-card-span">
              <div className="eyebrow">Access</div>
              <h4>{detail.access_reason}</h4>
              <div className="helper">
                Owner and subjects are separate records. Administrative access does not make the
                owner a reading subject.
              </div>
            </article>

            <article className="detail-card">
              <div className="eyebrow">Source</div>
              <h4>{detail.source.stable_source_id}</h4>
              <div className="detail-list">
                <div><span className="detail-label">Kind</span>{detail.source.source_kind}</div>
                <div><span className="detail-label">Locator</span><code>{detail.source.locator}</code></div>
                <div><span className="detail-label">SHA-256</span><code>{detail.source.content_sha256 ?? "--"}</code></div>
                <div><span className="detail-label">Size</span>{formatBytes(detail.source.byte_size)}</div>
                <div><span className="detail-label">Media</span>{detail.source.media_type ?? "--"}</div>
              </div>
            </article>

            <article className="detail-card">
              <div className="eyebrow">Import run</div>
              <h4>{detail.import_run.manifest_id}</h4>
              <div className="detail-list">
                <div><span className="detail-label">State</span>{detail.import_run.state}</div>
                <div><span className="detail-label">Schema</span>{detail.import_run.manifest_schema_version}</div>
                <div><span className="detail-label">Manifest SHA-256</span><code>{detail.import_run.manifest_sha256}</code></div>
                <div><span className="detail-label">Source root</span><code>{detail.import_run.source_root_locator}</code></div>
                <div><span className="detail-label">Started</span>{formatDateTime(detail.import_run.started_at)}</div>
              </div>
            </article>

            <article className="detail-card detail-card-span">
              <div className="eyebrow">Subjects and roles</div>
              <h4>{detail.reading.subjects.length} linked subject(s)</h4>
              <div className="detail-list">
                {detail.reading.subjects.map((subject) => (
                  <div key={`${subject.id}-${subject.role}`}>
                    <span className="detail-label">{subject.role}</span>
                    {subject.canonical_name}
                    <span className="helper">
                      {subject.aliases.length ? ` · aliases: ${subject.aliases.join(", ")}` : ""}
                    </span>
                  </div>
                ))}
              </div>
            </article>

            <article className="detail-card detail-card-span">
              <div className="eyebrow">Relationships</div>
              <h4>{detail.relationships.length} linked relationship(s)</h4>
              {detail.relationships.length ? detail.relationships.map((relationship) => (
                <div key={relationship.id} className="detail-list">
                  <div>
                    <span className="detail-label">{relationship.relationship_kind}</span>
                    {relationship.label ?? relationship.relationship_key}
                  </div>
                  {relationship.members.map((member) => (
                    <div key={`${relationship.id}-${member.subject_id}-${member.role}`}>
                      <span className="detail-label">{member.role}</span>
                      {member.canonical_name}
                    </div>
                  ))}
                </div>
              )) : <div className="helper">No relationship linked.</div>}
            </article>

            <article className="detail-card detail-card-span">
              <div className="eyebrow">Artifact metadata</div>
              <h4>{detail.artifacts.length} artifact(s) · locator display only</h4>
              <div className="detail-list">
                {detail.artifacts.map((artifact) => (
                  <div key={artifact.id}>
                    <span className="detail-label">{artifact.artifact_role}</span>
                    <strong>{artifact.artifact_key}</strong>
                    <div className="helper">
                      {artifact.storage_provider} · {formatBytes(artifact.byte_size)} · {artifact.availability_state}
                    </div>
                    <div><code>{artifact.object_locator}</code></div>
                    <div><code>sha256:{artifact.content_sha256}</code></div>
                  </div>
                ))}
              </div>
            </article>

            <article className="detail-card detail-card-span">
              <div className="eyebrow">Evidence</div>
              <h4>{detail.evidence.length} evidence record(s)</h4>
              <div className="detail-list">
                {detail.evidence.map((evidence) => (
                  <div key={evidence.id}>
                    <span className="detail-label">{evidence.review_state}</span>
                    <strong>{evidence.claim}</strong>
                    {evidence.excerpt ? <div className="helper">{evidence.excerpt}</div> : null}
                  </div>
                ))}
              </div>
            </article>

            <article className="detail-card detail-card-span">
              <div className="eyebrow">Editorial history</div>
              <h4>{detail.editorial_history.length} revision(s)</h4>
              <div className="detail-list">
                {detail.editorial_history.map((editorial) => (
                  <div key={editorial.id}>
                    <span className="detail-label">r{editorial.revision}</span>
                    <strong>{editorial.state}</strong> · {editorial.visibility} · {editorial.change_role}
                    <div className="helper">
                      {editorial.changed_by_email ?? "system"} · {formatDateTime(editorial.created_at)}
                      {editorial.is_current ? " · current" : ""}
                    </div>
                    {editorial.rationale ? <div>{editorial.rationale}</div> : null}
                  </div>
                ))}
              </div>
            </article>
          </div>
        ) : null}
      </DrawerSurface>
    </PageShell>
  );
}
