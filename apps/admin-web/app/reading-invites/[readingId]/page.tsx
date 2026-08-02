"use client";

import { useParams, useSearchParams } from "next/navigation";
import { useEffect, useState } from "react";
import { StatePanel } from "@/components/admin-state";
import { resolveLivingReadingInvitation } from "@/lib/api";
import {
  buildUraniaConversationUrl,
  wrapVerifiedHtmlForSandbox
} from "@/lib/living-reading-invite";
import type { LivingReadingInvitationResolution } from "@/types/admin";

export default function ReadingInvitationPage() {
  const params = useParams<{ readingId: string }>();
  const searchParams = useSearchParams();
  const token = searchParams.get("token") ?? "";
  const [reading, setReading] = useState<LivingReadingInvitationResolution | null>(null);
  const [unavailable, setUnavailable] = useState(false);

  useEffect(() => {
    let cancelled = false;
    if (!token) {
      return;
    }
    resolveLivingReadingInvitation(params.readingId, token)
      .then((resolved) => {
        if (!cancelled) setReading(resolved);
      })
      .catch(() => {
        if (!cancelled) setUnavailable(true);
      });
    return () => {
      cancelled = true;
    };
  }, [params.readingId, token]);

  if (!token || unavailable) {
    return (
      <main className="auth-page">
        <StatePanel
          variant="info"
          title="This invitation is unavailable"
          description="The link may have expired, been revoked, or belong to another reading."
        />
      </main>
    );
  }

  if (!reading) {
    return (
      <main className="auth-page">
        <StatePanel
          variant="loading"
          title="Opening your reading"
          description="Validating this private invitation."
        />
      </main>
    );
  }

  const artifactIsAvailable =
    reading.artifact.availability === "available" &&
    reading.artifact.content !== null &&
    reading.artifact.media_type !== null;
  const isHtml = artifactIsAvailable && reading.artifact.media_type === "text/html";
  const continueUrl = buildUraniaConversationUrl(
    reading.stable_reading_id,
    process.env.NEXT_PUBLIC_URANIA_URL
  );

  return (
    <main className="auth-page">
      <article className="panel filigree-frame">
        <div className="eyebrow">Living Reading</div>
        <h1>{reading.title}</h1>
        <p className="helper">
          {reading.reading_type} · {reading.language_tag}
          {reading.relationship_label ? ` · ${reading.relationship_label}` : ""}
        </p>
        {reading.summary ? <p>{reading.summary}</p> : null}

        {reading.subjects.length ? (
          <section>
            <h2>People in this reading</h2>
            <div className="detail-list">
              {reading.subjects.map((subject) => (
                <div key={`${subject.canonical_name}-${subject.role}`}>
                  <span className="detail-label">{subject.role}</span>
                  {subject.canonical_name}
                </div>
              ))}
            </div>
          </section>
        ) : null}

        {artifactIsAvailable ? (
          <section className="reading-publication" aria-label="Reading content">
            <div className="telemetry-caption">Your reading</div>
            {isHtml ? (
              <iframe
                className="reading-publication-frame"
                title={`${reading.title} reading content`}
                sandbox=""
                srcDoc={wrapVerifiedHtmlForSandbox(reading.artifact.content ?? "")}
              />
            ) : (
              <pre className="reading-publication-text">{reading.artifact.content}</pre>
            )}
          </section>
        ) : (
          <section className="state-banner state-banner-success" role="status">
            <div className="telemetry-caption">Publication availability</div>
            <div className="state-banner-title">Reading artifact is currently unavailable</div>
            <p className="helper">
              Your invitation is valid, but its published body could not be safely verified. No
              source file or internal storage location is exposed.
            </p>
          </section>
        )}

        <a className="reading-conversation-link" href={continueUrl}>
          Continue in conversation
        </a>
      </article>
    </main>
  );
}
