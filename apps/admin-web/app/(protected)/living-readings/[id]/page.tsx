"use client";

import Link from "next/link";
import { useParams } from "next/navigation";
import { useCallback, useEffect, useState } from "react";
import { ActionRail, MetricSurface } from "@/components/admin-primitives";
import { StateBanner, StatePanel } from "@/components/admin-state";
import { PageShell } from "@/components/page-shell";
import { getAuthToken } from "@/lib/auth";
import {
  ApiClientError,
  createAdminLivingReadingInvitation,
  getAdminLivingReading,
  getAdminLivingReadingInvitations,
  revokeAdminLivingReadingInvitation
} from "@/lib/api";
import type {
  AdminLivingReadingDetail,
  AdminLivingReadingInvitation
} from "@/types/admin";

function formatDateTime(value: string | null): string {
  if (!value) return "--";
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? "--" : date.toLocaleString();
}

function invitationState(invitation: AdminLivingReadingInvitation): string {
  if (invitation.revoked_at) return "revoked";
  return new Date(invitation.expires_at).getTime() <= Date.now() ? "expired" : "active";
}

export default function LivingReadingDetailPage() {
  const params = useParams<{ id: string }>();
  const readingId = params.id;
  const [detail, setDetail] = useState<AdminLivingReadingDetail | null>(null);
  const [invitations, setInvitations] = useState<AdminLivingReadingInvitation[]>([]);
  const [expiresInHours, setExpiresInHours] = useState(72);
  const [createdUrl, setCreatedUrl] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [working, setWorking] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const load = useCallback(async () => {
    const token = getAuthToken() ?? undefined;
    const [reading, invitePage] = await Promise.all([
      getAdminLivingReading(token, readingId),
      getAdminLivingReadingInvitations(token, readingId)
    ]);
    setDetail(reading);
    setInvitations(invitePage.items);
  }, [readingId]);

  useEffect(() => {
    let cancelled = false;
    async function run() {
      try {
        const token = getAuthToken() ?? undefined;
        const [reading, invitePage] = await Promise.all([
          getAdminLivingReading(token, readingId),
          getAdminLivingReadingInvitations(token, readingId)
        ]);
        if (!cancelled) {
          setDetail(reading);
          setInvitations(invitePage.items);
        }
      } catch (caught) {
        if (!cancelled) {
          setError(
            caught instanceof ApiClientError
              ? caught.payload?.error || caught.message
              : "Failed to load living reading"
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
  }, [readingId]);

  async function createInvitation() {
    setWorking(true);
    setError(null);
    try {
      const created = await createAdminLivingReadingInvitation(
        getAuthToken() ?? undefined,
        readingId,
        expiresInHours
      );
      const url = new URL(created.recipient_path, window.location.origin).toString();
      setCreatedUrl(url);
      await navigator.clipboard.writeText(url);
      await load();
    } catch (caught) {
      setError(
        caught instanceof ApiClientError
          ? caught.payload?.error || caught.message
          : "Failed to create invitation"
      );
    } finally {
      setWorking(false);
    }
  }

  async function revokeInvitation(invitationId: string) {
    setWorking(true);
    setError(null);
    try {
      await revokeAdminLivingReadingInvitation(
        getAuthToken() ?? undefined,
        readingId,
        invitationId
      );
      await load();
    } catch (caught) {
      setError(
        caught instanceof ApiClientError
          ? caught.payload?.error || caught.message
          : "Failed to revoke invitation"
      );
    } finally {
      setWorking(false);
    }
  }

  if (loading) {
    return (
      <StatePanel
        variant="loading"
        title="Loading archive detail"
        description="Resolving completion, publication, and invitation state."
      />
    );
  }

  return (
    <PageShell
      eyebrow="Living Archive"
      title={detail?.reading.title ?? "Reading unavailable"}
      summary={detail?.reading.summary ?? "No editorial summary has been recorded."}
      actions={
        <ActionRail label="Reading actions">
          <Link href="/living-readings">Back to archive</Link>
          <button
            type="button"
            className="primary"
            disabled={working || !detail}
            onClick={() => void createInvitation()}
          >
            {working ? "Working…" : "Invite"}
          </button>
        </ActionRail>
      }
    >
      {error ? <StateBanner variant="error" title={error} /> : null}
      {createdUrl ? (
        <StateBanner
          variant="success"
          title="Invitation copied"
          description="The bearer link is shown only now. Store and share it securely."
        />
      ) : null}

      {detail ? (
        <>
          <section className="metrics-grid">
            <MetricSurface
              label="Completion"
              value={detail.reading.editorial_state ?? "unreviewed"}
              detail={detail.reading.editorial_visibility ?? "no visibility set"}
            />
            <MetricSurface
              label="Publication"
              value={detail.reading.publication_availability.replaceAll("_", " ")}
              detail="Recipient content remains unavailable until a trusted publisher exists."
            />
            <MetricSurface
              label="Subjects"
              value={detail.reading.subjects.length}
              detail={detail.reading.subjects
                .map((subject) => subject.canonical_name)
                .join(", ") || "No subjects linked"}
            />
          </section>

          {detail.relationships.length ? (
            <article className="panel">
              <h3>Reading relationship</h3>
              <div className="detail-list">
                {detail.relationships.map((relationship) => (
                  <div key={relationship.id}>
                    <span className="detail-label">
                      {relationship.relationship_kind.replaceAll("_", " ")}
                    </span>
                    <strong>{relationship.label ?? "Relationship label pending review"}</strong>
                    <div className="inline-actions" aria-label="Relationship participants">
                      {relationship.members.map((member) => (
                        <span className="pill" key={`${member.subject_id}-${member.role}`}>
                          {member.canonical_name} · {member.role}
                        </span>
                      ))}
                    </div>
                  </div>
                ))}
              </div>
            </article>
          ) : null}

          <article className="panel">
            <h3>Create invitation</h3>
            <div className="panel-inline">
              <label>
                Expires in hours
                <input
                  type="number"
                  min={1}
                  max={720}
                  value={expiresInHours}
                  onChange={(event) => setExpiresInHours(Number(event.target.value))}
                />
              </label>
              <button
                type="button"
                className="primary"
                disabled={working}
                onClick={() => void createInvitation()}
              >
                Create and copy invite
              </button>
            </div>
            {createdUrl ? (
              <div className="detail-list invitation-link">
                <div>
                  <span className="detail-label">One-time link</span>
                  <code className="invitation-link-value">{createdUrl}</code>
                </div>
              </div>
            ) : null}
          </article>

          <article className="panel">
            <h3>Invitation history</h3>
            <div className="invitation-history" role="list">
              {invitations.length === 0 ? (
                <p className="helper">No invitation links have been created.</p>
              ) : invitations.map((invitation) => {
                const state = invitationState(invitation);
                return (
                  <article className="invitation-history-item" key={invitation.id} role="listitem">
                    <div>
                      <span className="detail-label">Status</span>
                      <span className="pill">{state}</span>
                    </div>
                    <div>
                      <span className="detail-label">Created</span>
                      <span>{formatDateTime(invitation.created_at)}</span>
                    </div>
                    <div>
                      <span className="detail-label">Expires</span>
                      <span>{formatDateTime(invitation.expires_at)}</span>
                    </div>
                    <div className="invitation-history-action">
                      <span className="detail-label">Action</span>
                      {state === "active" ? (
                        <button
                          type="button"
                          disabled={working}
                          onClick={() => void revokeInvitation(invitation.id)}
                        >
                          Revoke
                        </button>
                      ) : <span>--</span>}
                    </div>
                  </article>
                );
              })}
            </div>
          </article>
        </>
      ) : null}
    </PageShell>
  );
}
