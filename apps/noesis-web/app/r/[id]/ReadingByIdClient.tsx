"use client";

// ─── ReadingByIdClient — payload resolution + DepthReadingClient mount ──
// Tries (in order):
//   1. URL hash `#payload=<base64>` (zero-roundtrip, used when landing
//      redirected here with the full reading embedded)
//   2. Backend fetch `GET ${BACKEND_URL}/api/v1/readings/{id}`
//      (used for shareable URLs and page refresh)
//
// While loading: subtle pulsing dot + status text. On error: friendly
// "couldn't load" message + back-to-landing link.

import { useEffect, useState } from "react";
import {
  fetchPayloadById,
  tryDecodeHashPayload,
  type ReadingPayload,
} from "@/lib/integrated/payloadLoader";
import { buildSectionsFromPayload } from "@/lib/integrated/buildSectionsFromPayload";
import {
  cacheReading,
  getCachedReadingById,
  setPendingClaim,
  getLastReading,
} from "@/lib/integrated/readingCache";
import { isAuthenticated } from "@/lib/auth";
import { DepthReadingClient } from "@/depth-reading/DepthReadingClient";

interface ReadingByIdClientProps {
  readingId: string;
}

type LoadState =
  | { kind: "loading"; message: string }
  | { kind: "loaded"; payload: ReadingPayload }
  | { kind: "error"; message: string };

export default function ReadingByIdClient({ readingId }: ReadingByIdClientProps) {
  const [state, setState] = useState<LoadState>({
    kind: "loading",
    message: "Resolving the field…",
  });

  useEffect(() => {
    let cancelled = false;

    // 1. Try URL hash first — synchronous, no network round trip
    const inline = tryDecodeHashPayload();
    if (inline) {
      // Cache it so refresh / reload still works after the hash is consumed
      cacheReading(inline);
      if (!cancelled) setState({ kind: "loaded", payload: inline });
      return () => {
        cancelled = true;
      };
    }

    // 2. Try localStorage cache (this device may have generated it before)
    const cached = getCachedReadingById(readingId);
    if (cached) {
      if (!cancelled) setState({ kind: "loaded", payload: cached.payload });
      return () => {
        cancelled = true;
      };
    }

    // 3. Fall back to backend fetch (shareable URLs, cross-device access)
    setState({ kind: "loading", message: "Fetching the reading…" });
    fetchPayloadById(readingId).then((p) => {
      if (cancelled) return;
      if (p) {
        cacheReading(p);
        setState({ kind: "loaded", payload: p });
      } else
        setState({
          kind: "error",
          message:
            "We couldn't load this reading. The link may have expired or the backend is unreachable.",
        });
    });

    return () => {
      cancelled = true;
    };
  }, [readingId]);

  if (state.kind === "loading") {
    return <LoadingScreen message={state.message} />;
  }
  if (state.kind === "error") {
    return <ErrorScreen message={state.message} />;
  }

  // ─── Loaded ─────────────────────────────────────────────────────────
  const built = buildSectionsFromPayload(state.payload);

  // If the payload had nothing renderable, treat as error (instead of
  // showing an empty 0-plane scene which would look broken)
  if (built.sections.length === 0) {
    return (
      <ErrorScreen message="The reading came back empty. Try regenerating." />
    );
  }

  return (
    <>
      <DepthReadingClient
        sections={built.sections}
        proseBySection={built.proseBySection}
      />
      <SaveToAccountRibbon
        readingId={state.payload.reading_id ?? readingId}
      />
    </>
  );
}

// ─── SaveToAccountRibbon — the ONLY auth touchpoint in the read flow ───
// Subtle gold pill bottom-center on /r/[id]. Visible only when:
//   - user is anonymous (not signed in)
//   - reading isn't already claimed
// Clicking stashes a pendingClaim in localStorage + redirects to /auth.
// Post-OAuth, the callback handler picks up the claim and POSTs to
// the backend's /readings/{id}/claim endpoint.
function SaveToAccountRibbon({ readingId }: { readingId: string }) {
  const [show, setShow] = useState(false);
  const [hovered, setHovered] = useState(false);

  useEffect(() => {
    // Defer mount-time decision to client (avoids SSR mismatch)
    const authed = isAuthenticated();
    const cached = getLastReading();
    const alreadyClaimed = cached?.payload.reading_id === readingId && cached.claimed === true;
    setShow(!authed && !alreadyClaimed);
  }, [readingId]);

  if (!show) return null;

  const handleSave = () => {
    setPendingClaim(readingId);
    // Send to /auth with a hint about where to return after claiming
    const returnTo = encodeURIComponent(`/r/${readingId}`);
    window.location.assign(`/auth?next=${returnTo}&claim=1`);
  };

  return (
    <div
      style={{
        position: "fixed",
        bottom: "clamp(1rem, 3vh, 2rem)",
        left: "50%",
        transform: "translateX(-50%)",
        zIndex: 50,
        pointerEvents: "auto",
      }}
    >
      <button
        onClick={handleSave}
        onMouseEnter={() => setHovered(true)}
        onMouseLeave={() => setHovered(false)}
        style={{
          display: "flex",
          alignItems: "center",
          gap: "0.7rem",
          padding: "0.7rem 1.5rem",
          background: hovered
            ? "linear-gradient(135deg, #C5A017 0%, #C5A01788 100%)"
            : "rgba(7,11,29,0.78)",
          color: hovered ? "#070B1D" : "#C5A017",
          border: `1px solid ${hovered ? "#C5A017" : "rgba(197,160,23,0.45)"}`,
          borderRadius: "999px",
          fontFamily: "var(--font-mono, 'SF Mono', monospace)",
          fontSize: "0.7rem",
          letterSpacing: "0.3em",
          textTransform: "uppercase",
          cursor: "pointer",
          backdropFilter: "blur(10px)",
          boxShadow: hovered
            ? "0 12px 32px -10px rgba(197,160,23,0.65)"
            : "0 8px 24px -12px rgba(0,0,0,0.6)",
          transition: "all 0.25s cubic-bezier(0.2,0.7,0.2,1)",
        }}
        aria-label="Save this reading to your account"
      >
        <span aria-hidden="true" style={{ fontSize: "0.85rem" }}>⌬</span>
        <span>Save this reading</span>
      </button>
      <p
        style={{
          margin: "0.55rem 0 0",
          textAlign: "center",
          fontFamily: "var(--font-mono, 'SF Mono', monospace)",
          fontSize: "0.6rem",
          letterSpacing: "0.25em",
          color: "rgba(240,237,227,0.45)",
          textTransform: "uppercase",
        }}
      >
        opt-in · cross-device access · skip anytime
      </p>
    </div>
  );
}

// ─── Loading / error screens ────────────────────────────────────────────

function LoadingScreen({ message }: { message: string }) {
  return (
    <div
      style={{
        position: "fixed",
        inset: 0,
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        gap: "1.25rem",
        background: "var(--c-void, #070B1D)",
        color: "var(--c-parchment, #F0EDE3)",
        fontFamily: "var(--font-mono, 'SF Mono', monospace)",
        fontSize: "0.78rem",
        letterSpacing: "0.4em",
        textTransform: "uppercase",
      }}
    >
      <div
        aria-hidden="true"
        style={{
          width: "12px",
          height: "12px",
          borderRadius: "50%",
          background: "var(--c-gold, #C5A017)",
          animation: "depthReadingPulse 1.6s ease-in-out infinite",
        }}
      />
      <span style={{ opacity: 0.78 }}>{message}</span>
      <style>{`
        @keyframes depthReadingPulse {
          0%, 100% { transform: scale(0.7); opacity: 0.45; }
          50%      { transform: scale(1.2); opacity: 1.0; }
        }
      `}</style>
    </div>
  );
}

function ErrorScreen({ message }: { message: string }) {
  return (
    <div
      style={{
        position: "fixed",
        inset: 0,
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        gap: "1.5rem",
        padding: "2rem",
        background: "var(--c-void, #070B1D)",
        color: "var(--c-parchment, #F0EDE3)",
        textAlign: "center",
        fontFamily: "var(--font-body, 'Satoshi', sans-serif)",
      }}
    >
      <div
        style={{
          fontFamily: "var(--font-mono, 'SF Mono', monospace)",
          fontSize: "0.7rem",
          letterSpacing: "0.5em",
          textTransform: "uppercase",
          color: "var(--c-gold, #C5A017)",
          opacity: 0.78,
        }}
      >
        ∴ FIELD UNREACHABLE
      </div>
      <p
        style={{
          maxWidth: "44ch",
          fontSize: "clamp(1rem, 1.2vw, 1.15rem)",
          lineHeight: 1.55,
          color: "rgba(240,237,227,0.78)",
          margin: 0,
        }}
      >
        {message}
      </p>
      <a
        href="https://113.tryambakam.space/"
        style={{
          fontFamily: "var(--font-mono, 'SF Mono', monospace)",
          fontSize: "0.72rem",
          letterSpacing: "0.4em",
          textTransform: "uppercase",
          color: "var(--c-gold, #C5A017)",
          textDecoration: "none",
          padding: "0.6rem 1.4rem",
          border: "1px solid var(--c-gold, #C5A017)",
          borderRadius: "999px",
        }}
      >
        ← back to witness
      </a>
    </div>
  );
}
