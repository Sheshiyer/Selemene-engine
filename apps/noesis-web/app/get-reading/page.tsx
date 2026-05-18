"use client";

// ─── /get-reading — "The Threshold" conversational onboarding ───────────
// Replaces the traditional form-card with a step-by-step dyadic ritual.
// One threshold question at a time. A central sigil pulses, grows, and
// splits at the dyadic fork. Keyboard navigation (Enter / Esc), local-
// storage persistence per step, no native browser-control chrome.

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  DAILY_WITNESS_WORKFLOW_URL,
  INTEGRATED_READING_WORKFLOW_URL,
  WITNESS_LOCATIONS,
  buildDepthReadingPath,
  getWitnessLocation,
  readWitnessForm,
  writeWitnessForm,
  type WitnessLocation,
} from "@/lib/integrated/witnessAccess";
import { cacheReading } from "@/lib/integrated/readingCache";
import type { ReadingPayload } from "@/lib/integrated/payloadLoader";

type Step = 0 | 1 | 2 | 3 | 4 | 5;
type WorkflowKey = "daily" | "integrated";

interface ThresholdState {
  step: Step;
  name: string;
  birth_date: string; // YYYY-MM-DD
  birth_time: string; // HH:MM (24h) — optional
  location_key: string;
  submitting: WorkflowKey | null;
  error: string | null;
}

const INITIAL: ThresholdState = {
  step: 0,
  name: "",
  birth_date: "",
  birth_time: "",
  location_key: "",
  submitting: null,
  error: null,
};

// ─── Step copy (maximally in-character: Pichet = precision-scientist,
//      Aletheios = poet). Each step owned by ONE character or BOTH. ────
const STEP_PROMPT: Record<Step, string> = {
  0: "Step through.",
  1: "Name yourself. The field is listening for sound.",
  2: "Coordinates required. The date that anchored your arrival.",
  3: "And the hour? — sing it, or release it.",
  4: "Place the body. Latitude and longitude wait for the bone.",
  5: "We both ride either way. Pick the depth.",
};

const STEP_HELPER: Record<Step, string> = {
  0: "The dyad is waiting. Click anywhere to begin.",
  1: "Aletheios speaks first — Flow names you into being.",
  2: "Pichet asks now — Structure marks the moment.",
  3: "Aletheios returns — the hour is given, not chosen.",
  4: "Pichet roots you — coordinates wait for the bone.",
  5: "Both witnesses ride either way. The fork is between depths, not characters.",
};

/** Which witness owns each step — drives visual emphasis (active
 *  character lit + forward, the other dimmed + watching). */
type Speaker = "aletheios" | "pichet" | "both";
const STEP_SPEAKER: Record<Step, Speaker> = {
  0: "both",
  1: "aletheios",
  2: "pichet",
  3: "aletheios",
  4: "pichet",
  5: "both",
};

const SPEAKER_LABEL: Record<Speaker, string> = {
  aletheios: "ALETHEIOS",
  pichet: "PICHET",
  both: "ALETHEIOS + PICHET",
};

const SPEAKER_COLOR: Record<Speaker, string> = {
  aletheios: "#10B5A7", // Coherence Emerald
  pichet: "#C5A017",    // Sacred Gold
  both: "#F0EDE3",      // Parchment (joined)
};

// ─── Page ───────────────────────────────────────────────────────────────
export default function GetReadingPage() {
  const [state, setState] = useState<ThresholdState>(INITIAL);

  // Hydrate from localStorage on mount — returning visitors skip a step
  useEffect(() => {
    const stored = readWitnessForm();
    if (!stored) return;
    setState((s) => ({
      ...s,
      name: stored.name ?? "",
      birth_date: stored.birth_date ?? "",
      birth_time: stored.birth_time ?? "",
      location_key: stored.location_key ?? "",
    }));
  }, []);

  // Persist relevant fields whenever they change
  useEffect(() => {
    writeWitnessForm({
      name: state.name,
      birth_date: state.birth_date,
      birth_time: state.birth_time,
      location_key: state.location_key,
    });
  }, [state.name, state.birth_date, state.birth_time, state.location_key]);

  const setStep = useCallback((step: Step) => {
    setState((s) => ({ ...s, step, error: null }));
  }, []);

  const advance = useCallback(() => {
    setState((s) => {
      const next = Math.min(5, s.step + 1) as Step;
      // Field-level validation per step
      if (s.step === 2 && !s.birth_date) {
        return { ...s, error: "The field needs a date." };
      }
      if (s.step === 4 && !s.location_key) {
        return { ...s, error: "The field needs a place." };
      }
      return { ...s, step: next, error: null };
    });
  }, []);

  const retreat = useCallback(() => {
    setState((s) => ({
      ...s,
      step: Math.max(0, s.step - 1) as Step,
      error: null,
    }));
  }, []);

  // ─── Submit ──────────────────────────────────────────────────────────
  const submit = useCallback(
    async (workflow: WorkflowKey) => {
      const location = getWitnessLocation(state.location_key);
      if (!location) {
        setState((s) => ({ ...s, error: "The field needs a place.", step: 4 }));
        return;
      }
      if (!state.birth_date) {
        setState((s) => ({ ...s, error: "The field needs a date.", step: 2 }));
        return;
      }

      setState((s) => ({ ...s, submitting: workflow, error: null }));

      const payload: Record<string, unknown> = {
        birth_data: {
          date: state.birth_date,
          timezone: location.timezone,
          latitude: location.latitude,
          longitude: location.longitude,
        },
      };
      if (state.birth_time) (payload.birth_data as Record<string, unknown>).time = state.birth_time;
      if (state.name) (payload.birth_data as Record<string, unknown>).name = state.name;

      try {
        const url = workflow === "integrated"
          ? INTEGRATED_READING_WORKFLOW_URL
          : DAILY_WITNESS_WORKFLOW_URL;
        const res = await fetch(url, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify(payload),
        });
        const body = (await res.json().catch(() => null)) as
          | (Record<string, unknown> & { reading_id?: string })
          | null;
        const hasContent =
          body?.witness_layer ||
          (Array.isArray((body as { passes?: unknown })?.passes) &&
            ((body as { passes: unknown[] }).passes.length ?? 0) > 0);
        if (!res.ok || !hasContent) {
          const msg =
            (body as { error?: string; message?: string })?.error ||
            (body as { message?: string })?.message ||
            `The field returned ${res.status}.`;
          throw new Error(msg);
        }
        // Cache the reading anonymously — no auth required to keep it.
        // localStorage stores last reading + history of 10 on this
        // device. Stays accessible even after redirect / refresh /
        // browser restart, with no backend dependency.
        cacheReading((body ?? {}) as ReadingPayload);
        const dest = buildDepthReadingPath(body ?? {});
        window.location.assign(dest);
      } catch (e) {
        const msg = e instanceof Error ? e.message : "The field withheld.";
        setState((s) => ({ ...s, submitting: null, error: msg }));
      }
    },
    [state.birth_date, state.birth_time, state.location_key, state.name],
  );

  // ─── Keyboard nav ────────────────────────────────────────────────────
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      // Don't hijack typing in inputs
      const target = e.target as HTMLElement | null;
      if (target && (target.tagName === "INPUT" || target.tagName === "SELECT" || target.tagName === "TEXTAREA")) {
        // Allow Enter to advance from inputs (don't preventDefault — let the form handle submit/enter)
        if (e.key === "Enter") {
          e.preventDefault();
          if (state.step < 5) advance();
        }
        if (e.key === "Escape") {
          e.preventDefault();
          retreat();
        }
        return;
      }
      if (e.key === "Enter" || e.key === " ") {
        if (state.step === 0) {
          e.preventDefault();
          setStep(1);
        } else if (state.step < 5) {
          e.preventDefault();
          advance();
        }
      }
      if (e.key === "Escape" || e.key === "Backspace") {
        e.preventDefault();
        retreat();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [state.step, advance, retreat, setStep]);

  return (
    <div
      style={s.page}
      onClick={() => {
        if (state.step === 0) setStep(1);
      }}
    >
      <DyadChamber step={state.step} submitting={state.submitting} />

      <div style={s.stage} onClick={(e) => e.stopPropagation()}>
        <StepIndicator step={state.step} onJump={(n) => setStep(n)} />

        <div style={s.prompt}>
          {/* Voice attribution eyebrow — who's speaking this step */}
          <p
            style={{
              ...s.speakerEyebrow,
              color: SPEAKER_COLOR[STEP_SPEAKER[state.step]],
            }}
          >
            {SPEAKER_LABEL[STEP_SPEAKER[state.step]]}:
          </p>
          <h1 style={s.question}>{STEP_PROMPT[state.step]}</h1>
          <p style={s.helper}>{STEP_HELPER[state.step]}</p>
        </div>

        <div style={s.field}>
          {state.step === 1 && (
            <NameInput
              value={state.name}
              onChange={(name) => setState((s) => ({ ...s, name }))}
              onAdvance={advance}
            />
          )}
          {state.step === 2 && (
            <DateInput
              value={state.birth_date}
              onChange={(d) => setState((s) => ({ ...s, birth_date: d }))}
              onAdvance={advance}
            />
          )}
          {state.step === 3 && (
            <TimeInput
              value={state.birth_time}
              onChange={(t) => setState((s) => ({ ...s, birth_time: t }))}
              onSkip={advance}
              onAdvance={advance}
            />
          )}
          {state.step === 4 && (
            <LocationPicker
              value={state.location_key}
              onChange={(k) => setState((s) => ({ ...s, location_key: k }))}
              onAdvance={advance}
            />
          )}
          {state.step === 5 && (
            <DyadicFork submitting={state.submitting} onPick={submit} />
          )}
        </div>

        {/* Error line — phrased in-character */}
        {state.error && <p style={s.error}>{state.error}</p>}

        {/* Footer nav: back + (for non-final) continue pill */}
        {state.step > 0 && state.step < 5 && (
          <div style={s.nav}>
            <button onClick={retreat} style={s.navBack} type="button">
              ← back
            </button>
            <button onClick={advance} style={s.navForward} type="button">
              continue →
            </button>
          </div>
        )}
        {state.step === 5 && (
          <div style={s.nav}>
            <button onClick={retreat} style={s.navBack} type="button">
              ← back
            </button>
          </div>
        )}
      </div>

      {/* No sign-in trap at step 0 — readings are anonymous by default.
          Power-user account access is reachable from /readings or
          /engines (visible only AFTER a reading exists). */}

      {/* Global styles + keyframes */}
      <style jsx global>{`
        @keyframes thresholdPulse {
          0%, 100% { transform: scale(0.98); filter: drop-shadow(0 0 18px rgba(197,160,23,0.45)); }
          50%      { transform: scale(1.04); filter: drop-shadow(0 0 36px rgba(197,160,23,0.80)); }
        }
        @keyframes thresholdRingDrift {
          from { transform: rotate(0deg); }
          to   { transform: rotate(360deg); }
        }
        @keyframes thresholdStepFade {
          from { opacity: 0; transform: translateY(8px); }
          to   { opacity: 1; transform: translateY(0); }
        }
        .threshold-step-enter { animation: thresholdStepFade 600ms cubic-bezier(0.2,0.7,0.2,1) both; }
      `}</style>
    </div>
  );
}

// ─── DyadChamber — both witness characters always visible ──────────────
// Pichet on the left, Aletheios on the right. The character owning the
// current step is fully lit + slightly forward; the other dims back
// + watches. At step 5 (dyadic fork), BOTH are equally lit. At step 0,
// both are dim but symmetric — the dyad awakes.
function DyadChamber({
  step,
  submitting,
}: {
  step: Step;
  submitting: WorkflowKey | null;
}) {
  const speaker = STEP_SPEAKER[step];
  const pichetActive = speaker === "pichet" || speaker === "both";
  const aletheiosActive = speaker === "aletheios" || speaker === "both";

  // Submission state pulses brightness on whichever character corresponds
  // to the chosen lens; for "both" we accelerate both.
  const isSubmitting = submitting !== null;

  return (
    <div style={s.sigilStage} aria-hidden="true">
      {/* Faint orbital sigil between the two witnesses — present but subtle. */}
      <svg
        viewBox="0 0 200 200"
        style={{
          position: "absolute",
          width: "clamp(360px, 50vmin, 720px)",
          height: "clamp(360px, 50vmin, 720px)",
          opacity: 0.25,
          pointerEvents: "none",
        }}
      >
        <g style={{ animation: "thresholdRingDrift 120s linear infinite", transformOrigin: "100px 100px" }}>
          <circle cx="100" cy="100" r="94" fill="none" stroke="#C5A017" strokeOpacity="0.4" strokeWidth="0.4" />
          <circle cx="100" cy="100" r="78" fill="none" stroke="#10B5A7" strokeOpacity="0.4" strokeWidth="0.4" />
        </g>
      </svg>

      <WitnessFigure
        side="left"
        name="pichet"
        active={pichetActive}
        submitting={isSubmitting && (speaker === "pichet" || speaker === "both")}
      />
      <WitnessFigure
        side="right"
        name="aletheios"
        active={aletheiosActive}
        submitting={isSubmitting && (speaker === "aletheios" || speaker === "both")}
      />
    </div>
  );
}

function WitnessFigure({
  side,
  name,
  active,
  submitting,
}: {
  side: "left" | "right";
  name: "pichet" | "aletheios";
  active: boolean;
  submitting: boolean;
}) {
  // Active witness: full opacity + subtle forward step + soft glow.
  // Inactive witness: dim (~30%) + subtle backward step + watching toward center.
  const opacity = active ? 1 : 0.35;
  const translateX = active
    ? (side === "left" ? "8px" : "-8px")
    : (side === "left" ? "-12px" : "12px");
  const glowColor = name === "pichet" ? "rgba(197,160,23,0.4)" : "rgba(16,181,167,0.4)";
  const filter = active ? `drop-shadow(0 0 24px ${glowColor})` : "saturate(0.4) brightness(0.6)";

  return (
    <div
      style={{
        position: "absolute",
        top: "50%",
        [side]: "clamp(0px, 4vw, 4rem)",
        transform: `translateY(-50%) translateX(${translateX})`,
        height: "clamp(50vh, 75vh, 90vh)",
        width: "auto",
        aspectRatio: "1 / 1.4",
        backgroundImage: `url(/depth-reading/characters/${name}-front.png)`,
        backgroundSize: "contain",
        backgroundRepeat: "no-repeat",
        backgroundPosition: side === "left" ? "left center" : "right center",
        opacity,
        filter,
        transition:
          "opacity 700ms cubic-bezier(0.2,0.7,0.2,1), transform 700ms cubic-bezier(0.2,0.7,0.2,1), filter 700ms cubic-bezier(0.2,0.7,0.2,1)",
        pointerEvents: "none",
        animation: submitting ? "thresholdPulse 1.2s ease-in-out infinite" : undefined,
      }}
    />
  );
}

// ─── StepIndicator — 5 sigil tokens, gold for Pichet's steps,
//      emerald for Aletheios's, trinity-dot for step 5 (both). ─────────
function StepIndicator({
  step,
  onJump,
}: {
  step: Step;
  onJump: (n: Step) => void;
}) {
  // Step 1: Aletheios (vesica), 2: Pichet (hex), 3: Aletheios (vesica),
  // 4: Pichet (hex), 5: both (trinity-dot)
  const tokens: Array<{ stepNum: Step; speaker: Speaker; symbol: "vesica" | "hex" | "trinity" }> = [
    { stepNum: 1 as Step, speaker: "aletheios", symbol: "vesica" },
    { stepNum: 2 as Step, speaker: "pichet",    symbol: "hex" },
    { stepNum: 3 as Step, speaker: "aletheios", symbol: "vesica" },
    { stepNum: 4 as Step, speaker: "pichet",    symbol: "hex" },
    { stepNum: 5 as Step, speaker: "both",      symbol: "trinity" },
  ];

  return (
    <div style={s.progress} role="progressbar" aria-valuemin={1} aria-valuemax={5} aria-valuenow={step}>
      {tokens.map((t) => {
        const isActive = t.stepNum === step;
        const isPast = t.stepNum < step;
        const color = SPEAKER_COLOR[t.speaker];
        const intensity = isActive ? 1 : isPast ? 0.55 : 0.18;
        return (
          <button
            key={t.stepNum}
            type="button"
            onClick={() => onJump(t.stepNum)}
            aria-label={`Step ${t.stepNum}`}
            title={SPEAKER_LABEL[t.speaker]}
            style={{
              ...s.tokenButton,
              transform: isActive ? "scale(1.25)" : "scale(1)",
              filter: isActive ? `drop-shadow(0 0 8px ${color})` : undefined,
            }}
          >
            <SigilToken symbol={t.symbol} color={color} opacity={intensity} />
          </button>
        );
      })}
    </div>
  );
}

function SigilToken({
  symbol,
  color,
  opacity,
}: {
  symbol: "vesica" | "hex" | "trinity";
  color: string;
  opacity: number;
}) {
  if (symbol === "hex") {
    return (
      <svg width="14" height="16" viewBox="0 0 14 16" aria-hidden="true">
        <polygon
          points="7,1 13,4.5 13,11.5 7,15 1,11.5 1,4.5"
          fill="none"
          stroke={color}
          strokeWidth="1.2"
          opacity={opacity}
        />
      </svg>
    );
  }
  if (symbol === "vesica") {
    return (
      <svg width="16" height="16" viewBox="0 0 16 16" aria-hidden="true">
        <circle cx="5.5" cy="8" r="4.2" fill="none" stroke={color} strokeWidth="1.0" opacity={opacity} />
        <circle cx="10.5" cy="8" r="4.2" fill="none" stroke={color} strokeWidth="1.0" opacity={opacity} />
      </svg>
    );
  }
  // trinity dot — three small dots in a triangle
  return (
    <svg width="14" height="14" viewBox="0 0 14 14" aria-hidden="true">
      <circle cx="7"  cy="3"   r="1.4" fill={color} opacity={opacity} />
      <circle cx="3"  cy="10"  r="1.4" fill={color} opacity={opacity} />
      <circle cx="11" cy="10"  r="1.4" fill={color} opacity={opacity} />
    </svg>
  );
}

// ─── NameInput ──────────────────────────────────────────────────────────
function NameInput({
  value,
  onChange,
  onAdvance,
}: {
  value: string;
  onChange: (s: string) => void;
  onAdvance: () => void;
}) {
  const ref = useRef<HTMLInputElement>(null);
  useEffect(() => {
    ref.current?.focus();
  }, []);
  return (
    <input
      ref={ref}
      type="text"
      value={value}
      onChange={(e) => onChange(e.target.value)}
      onKeyDown={(e) => {
        if (e.key === "Enter") {
          e.preventDefault();
          onAdvance();
        }
      }}
      placeholder="(or leave blank — the field still finds you)"
      style={s.bigInput}
      autoComplete="given-name"
      spellCheck={false}
    />
  );
}

// ─── DateInput — 3 separate styled numeric fields (day / month / year) ──
function DateInput({
  value,
  onChange,
  onAdvance,
}: {
  value: string;
  onChange: (v: string) => void;
  onAdvance: () => void;
}) {
  // Parse YYYY-MM-DD into parts
  const parts = value.match(/^(\d{4})-(\d{2})-(\d{2})$/);
  const initYear = parts?.[1] ?? "";
  const initMonth = parts?.[2] ?? "";
  const initDay = parts?.[3] ?? "";
  const [d, setD] = useState(initDay);
  const [m, setM] = useState(initMonth);
  const [y, setY] = useState(initYear);

  // Compose YYYY-MM-DD when all three are valid
  useEffect(() => {
    if (/^\d{1,2}$/.test(d) && /^\d{1,2}$/.test(m) && /^\d{4}$/.test(y)) {
      const dd = d.padStart(2, "0");
      const mm = m.padStart(2, "0");
      if (parseInt(dd, 10) >= 1 && parseInt(dd, 10) <= 31 && parseInt(mm, 10) >= 1 && parseInt(mm, 10) <= 12) {
        onChange(`${y}-${mm}-${dd}`);
      }
    }
  }, [d, m, y, onChange]);

  const inputs = useMemo(
    () =>
      [
        { value: d, setValue: setD, label: "day", placeholder: "DD", max: 2 },
        { value: m, setValue: setM, label: "month", placeholder: "MM", max: 2 },
        { value: y, setValue: setY, label: "year", placeholder: "YYYY", max: 4 },
      ] as const,
    [d, m, y],
  );

  return (
    <div style={s.dateRow}>
      {inputs.map((it, i) => (
        <div key={it.label} style={s.dateCell}>
          <input
            type="text"
            inputMode="numeric"
            pattern="[0-9]*"
            value={it.value}
            maxLength={it.max}
            onChange={(e) => it.setValue(e.target.value.replace(/\D/g, ""))}
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                e.preventDefault();
                onAdvance();
              }
            }}
            placeholder={it.placeholder}
            style={s.dateInput}
            autoFocus={i === 0 && !it.value}
          />
          <span style={s.dateLabel}>{it.label}</span>
        </div>
      ))}
    </div>
  );
}

// ─── TimeInput ──────────────────────────────────────────────────────────
function TimeInput({
  value,
  onChange,
  onSkip,
  onAdvance,
}: {
  value: string;
  onChange: (v: string) => void;
  onSkip: () => void;
  onAdvance: () => void;
}) {
  const parts = value.match(/^(\d{2}):(\d{2})$/);
  const [h, setH] = useState(parts?.[1] ?? "");
  const [m, setM] = useState(parts?.[2] ?? "");

  useEffect(() => {
    if (/^\d{1,2}$/.test(h) && /^\d{1,2}$/.test(m)) {
      const hh = h.padStart(2, "0");
      const mm = m.padStart(2, "0");
      if (parseInt(hh, 10) <= 23 && parseInt(mm, 10) <= 59) {
        onChange(`${hh}:${mm}`);
      }
    } else if (!h && !m) {
      onChange("");
    }
  }, [h, m, onChange]);

  return (
    <div style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: "1.5rem" }}>
      <div style={s.dateRow}>
        <div style={s.dateCell}>
          <input
            type="text"
            inputMode="numeric"
            pattern="[0-9]*"
            value={h}
            maxLength={2}
            onChange={(e) => setH(e.target.value.replace(/\D/g, ""))}
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                e.preventDefault();
                onAdvance();
              }
            }}
            placeholder="HH"
            style={s.dateInput}
            autoFocus
          />
          <span style={s.dateLabel}>hour</span>
        </div>
        <div style={s.dateCell}>
          <input
            type="text"
            inputMode="numeric"
            pattern="[0-9]*"
            value={m}
            maxLength={2}
            onChange={(e) => setM(e.target.value.replace(/\D/g, ""))}
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                e.preventDefault();
                onAdvance();
              }
            }}
            placeholder="MM"
            style={s.dateInput}
          />
          <span style={s.dateLabel}>minute</span>
        </div>
      </div>
      <button type="button" onClick={onSkip} style={s.skipPill}>
        I don't know my hour
      </button>
    </div>
  );
}

// ─── LocationPicker — typeahead with constellation results ──────────────
function LocationPicker({
  value,
  onChange,
  onAdvance,
}: {
  value: string;
  onChange: (key: string) => void;
  onAdvance: () => void;
}) {
  const [query, setQuery] = useState(() => {
    const found = WITNESS_LOCATIONS.find((l) => l.key === value);
    return found?.label ?? "";
  });
  const ref = useRef<HTMLInputElement>(null);
  useEffect(() => {
    ref.current?.focus();
  }, []);

  const matches = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return WITNESS_LOCATIONS.slice(0, 8);
    return WITNESS_LOCATIONS.filter((l) =>
      l.label.toLowerCase().includes(q) || l.group.toLowerCase().includes(q),
    ).slice(0, 8);
  }, [query]);

  const pick = (l: WitnessLocation) => {
    onChange(l.key);
    setQuery(l.label);
    setTimeout(onAdvance, 200);
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", alignItems: "center", gap: "1rem", width: "100%" }}>
      <input
        ref={ref}
        type="text"
        value={query}
        onChange={(e) => {
          setQuery(e.target.value);
          onChange(""); // clear selection while typing
        }}
        onKeyDown={(e) => {
          if (e.key === "Enter" && matches[0]) {
            e.preventDefault();
            pick(matches[0]);
          }
        }}
        placeholder="type a city…"
        style={s.bigInput}
        spellCheck={false}
      />
      <ul style={s.locList}>
        {matches.map((l) => (
          <li key={l.key}>
            <button
              type="button"
              onClick={() => pick(l)}
              style={{
                ...s.locItem,
                ...(value === l.key ? s.locItemActive : {}),
              }}
            >
              <span style={s.locDot} />
              <span style={s.locLabel}>{l.label}</span>
              <span style={s.locTz}>{l.timezone.replace("_", " ")}</span>
            </button>
          </li>
        ))}
        {matches.length === 0 && (
          <li>
            <span style={{ ...s.locItem, color: "rgba(240,237,227,0.5)" }}>
              the field doesn't recognize that place yet
            </span>
          </li>
        )}
      </ul>
    </div>
  );
}

// ─── DyadicFork — final step. Two cards, daily vs integrated. ───────────
function DyadicFork({
  submitting,
  onPick,
}: {
  submitting: WorkflowKey | null;
  onPick: (w: WorkflowKey) => void;
}) {
  return (
    <div style={s.forkRow}>
      <ForkCard
        accent="#C5A017"
        label="ALETHEIOS · DAILY"
        title="One witness, one pass."
        meta="~3 minutes"
        active={submitting === "daily"}
        disabled={submitting !== null}
        onClick={() => onPick("daily")}
      />
      <ForkCard
        accent="#10B5A7"
        label="PICHET · INTEGRATED"
        title="Sixteen mirrors, full synthesis."
        meta="~30 seconds"
        active={submitting === "integrated"}
        disabled={submitting !== null}
        onClick={() => onPick("integrated")}
      />
    </div>
  );
}

function ForkCard({
  accent,
  label,
  title,
  meta,
  active,
  disabled,
  onClick,
}: {
  accent: string;
  label: string;
  title: string;
  meta: string;
  active: boolean;
  disabled: boolean;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      disabled={disabled}
      style={{
        ...s.forkCard,
        borderColor: active ? accent : `${accent}55`,
        boxShadow: active
          ? `0 0 38px ${accent}88, inset 0 0 28px ${accent}22`
          : `0 12px 36px -22px ${accent}aa`,
        opacity: disabled && !active ? 0.4 : 1,
        cursor: disabled ? "default" : "pointer",
      }}
    >
      <span style={{ ...s.forkLabel, color: accent }}>{label}</span>
      <h3 style={s.forkTitle}>{title}</h3>
      <span style={s.forkMeta}>{meta}</span>
      {active && (
        <span style={{ ...s.forkLoading, color: accent }}>
          ◌ composing the field…
        </span>
      )}
    </button>
  );
}

// ─── Styles ─────────────────────────────────────────────────────────────
const s: Record<string, React.CSSProperties> = {
  page: {
    position: "fixed",
    inset: 0,
    overflow: "hidden",
    background: "radial-gradient(ellipse at 30% 20%, #2D005066 0%, #070B1D 70%), #070B1D",
    color: "#F0EDE3",
    display: "flex",
    flexDirection: "column",
    alignItems: "center",
    justifyContent: "center",
    padding: "clamp(1rem, 4vh, 3rem)",
  },
  sigilStage: {
    position: "absolute",
    inset: 0,
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    pointerEvents: "none",
  },
  sigilDrifting: {
    position: "absolute",
    transition: "width 1.2s cubic-bezier(0.2,0.7,0.2,1), height 1.2s cubic-bezier(0.2,0.7,0.2,1), opacity 0.6s ease",
  },
  coreOrb: {
    position: "absolute",
    width: "clamp(80px, 12vmin, 140px)",
    height: "clamp(80px, 12vmin, 140px)",
    borderRadius: "50%",
    transition: "opacity 0.6s ease",
  },
  dyadHalf: {
    position: "absolute",
    top: "50%",
    transform: "translateY(-50%)",
  },
  stage: {
    position: "relative",
    zIndex: 3, // above the witness figures
    width: "min(560px, 92vw)",
    display: "flex",
    flexDirection: "column",
    alignItems: "center",
    gap: "clamp(1.25rem, 2.5vh, 2rem)",
    // Add backdrop blur behind the center column so witness figures
    // don't visually fight with the dialog text on narrow viewports
    background:
      "radial-gradient(ellipse at center, rgba(7,11,29,0.55) 0%, rgba(7,11,29,0.0) 70%)",
    padding: "clamp(1rem, 3vh, 2.5rem)",
    borderRadius: "24px",
  },
  progress: {
    display: "flex",
    gap: "0.85rem",
    alignItems: "center",
  },
  tokenButton: {
    background: "transparent",
    border: "none",
    padding: "0.25rem",
    cursor: "pointer",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    transition: "all 0.3s ease",
  },
  speakerEyebrow: {
    margin: 0,
    fontFamily: "var(--font-mono, 'SF Mono', monospace)",
    fontSize: "clamp(0.6rem, 0.72vw, 0.72rem)",
    letterSpacing: "0.4em",
    textTransform: "uppercase",
    fontWeight: 600,
  },
  prompt: {
    display: "flex",
    flexDirection: "column",
    alignItems: "center",
    gap: "0.75rem",
    textAlign: "center",
    animation: "thresholdStepFade 600ms cubic-bezier(0.2,0.7,0.2,1) both",
  },
  helper: {
    margin: 0,
    fontFamily: "var(--font-mono, 'SF Mono', monospace)",
    fontSize: "clamp(0.62rem, 0.78vw, 0.78rem)",
    letterSpacing: "0.32em",
    textTransform: "uppercase",
    color: "rgba(240,237,227,0.55)",
    maxWidth: "44ch",
    lineHeight: 1.6,
  },
  question: {
    margin: 0,
    fontFamily: "var(--font-display, 'Panchang', serif)",
    fontVariationSettings: "'wght' 720",
    fontSize: "clamp(1.7em, min(4.5vw, 7vh), 3em)",
    lineHeight: 1.06,
    letterSpacing: "-0.012em",
    color: "#F0EDE3",
    maxWidth: "20ch",
  },
  field: {
    width: "100%",
    display: "flex",
    flexDirection: "column",
    alignItems: "center",
    gap: "1rem",
    animation: "thresholdStepFade 700ms cubic-bezier(0.2,0.7,0.2,1) 100ms both",
  },
  bigInput: {
    width: "min(520px, 100%)",
    padding: "0.85rem 0",
    background: "transparent",
    border: "none",
    borderBottom: "1px solid rgba(240,237,227,0.18)",
    color: "#F0EDE3",
    fontFamily: "var(--font-display, 'Panchang', serif)",
    fontSize: "clamp(1.1rem, 2.2vw, 1.6rem)",
    fontVariationSettings: "'wght' 500",
    textAlign: "center",
    outline: "none",
    letterSpacing: "0.01em",
  },
  dateRow: {
    display: "flex",
    gap: "clamp(0.75rem, 2vw, 1.75rem)",
    alignItems: "flex-end",
  },
  dateCell: {
    display: "flex",
    flexDirection: "column",
    alignItems: "center",
    gap: "0.5rem",
  },
  dateInput: {
    width: "5.5rem",
    padding: "0.7rem 0",
    background: "transparent",
    border: "none",
    borderBottom: "1px solid rgba(240,237,227,0.18)",
    color: "#F0EDE3",
    fontFamily: "var(--font-display, 'Panchang', serif)",
    fontSize: "clamp(1.6rem, 3vw, 2.2rem)",
    fontVariationSettings: "'wght' 640",
    textAlign: "center",
    outline: "none",
    letterSpacing: "0.04em",
  },
  dateLabel: {
    fontFamily: "var(--font-mono, 'SF Mono', monospace)",
    fontSize: "0.62rem",
    letterSpacing: "0.32em",
    textTransform: "uppercase",
    color: "rgba(240,237,227,0.5)",
  },
  skipPill: {
    padding: "0.55rem 1.1rem",
    background: "transparent",
    border: "1px solid rgba(197,160,23,0.4)",
    color: "#C5A017",
    borderRadius: "999px",
    fontFamily: "var(--font-mono, 'SF Mono', monospace)",
    fontSize: "0.66rem",
    letterSpacing: "0.3em",
    textTransform: "uppercase",
    cursor: "pointer",
    transition: "all 0.2s ease",
  },
  locList: {
    listStyle: "none",
    margin: 0,
    padding: 0,
    width: "min(520px, 100%)",
    maxHeight: "clamp(180px, 35vh, 280px)",
    overflowY: "auto",
    display: "flex",
    flexDirection: "column",
    gap: "0.25rem",
  },
  locItem: {
    display: "flex",
    alignItems: "center",
    gap: "0.75rem",
    width: "100%",
    padding: "0.7rem 0.9rem",
    background: "transparent",
    border: "1px solid transparent",
    borderRadius: "10px",
    color: "rgba(240,237,227,0.85)",
    fontFamily: "var(--font-body, 'Satoshi', sans-serif)",
    fontSize: "0.92rem",
    cursor: "pointer",
    textAlign: "left",
    transition: "all 0.15s ease",
  },
  locItemActive: {
    background: "rgba(197,160,23,0.12)",
    borderColor: "rgba(197,160,23,0.4)",
  },
  locDot: {
    width: "5px",
    height: "5px",
    borderRadius: "50%",
    background: "#C5A017",
    flexShrink: 0,
  },
  locLabel: {
    flex: 1,
  },
  locTz: {
    fontFamily: "var(--font-mono, 'SF Mono', monospace)",
    fontSize: "0.66rem",
    letterSpacing: "0.16em",
    color: "rgba(240,237,227,0.5)",
  },
  forkRow: {
    display: "flex",
    gap: "clamp(0.75rem, 2vw, 1.5rem)",
    width: "100%",
    justifyContent: "center",
    flexWrap: "wrap",
  },
  forkCard: {
    flex: "1 1 200px",
    maxWidth: "260px",
    minHeight: "200px",
    padding: "1.4rem 1.2rem",
    background: "rgba(7,11,29,0.55)",
    border: "1px solid",
    borderRadius: "16px",
    backdropFilter: "blur(10px)",
    display: "flex",
    flexDirection: "column",
    alignItems: "flex-start",
    gap: "0.5rem",
    textAlign: "left",
    transition: "all 0.25s ease",
  },
  forkLabel: {
    fontFamily: "var(--font-mono, 'SF Mono', monospace)",
    fontSize: "0.66rem",
    letterSpacing: "0.32em",
    textTransform: "uppercase",
  },
  forkTitle: {
    margin: "0.5rem 0 auto",
    fontFamily: "var(--font-display, 'Panchang', serif)",
    fontSize: "clamp(1.05rem, 1.6vw, 1.3rem)",
    fontVariationSettings: "'wght' 600",
    color: "#F0EDE3",
    lineHeight: 1.2,
  },
  forkMeta: {
    fontFamily: "var(--font-mono, 'SF Mono', monospace)",
    fontSize: "0.66rem",
    letterSpacing: "0.18em",
    color: "rgba(240,237,227,0.55)",
  },
  forkLoading: {
    fontFamily: "var(--font-mono, 'SF Mono', monospace)",
    fontSize: "0.66rem",
    letterSpacing: "0.18em",
    marginTop: "0.5rem",
  },
  nav: {
    display: "flex",
    gap: "1.5rem",
    alignItems: "center",
  },
  navBack: {
    background: "transparent",
    border: "none",
    color: "rgba(240,237,227,0.55)",
    fontFamily: "var(--font-mono, 'SF Mono', monospace)",
    fontSize: "0.66rem",
    letterSpacing: "0.3em",
    textTransform: "uppercase",
    cursor: "pointer",
    padding: "0.4rem 0.6rem",
    transition: "color 0.2s ease",
  },
  navForward: {
    padding: "0.5rem 1.2rem",
    background: "transparent",
    border: "1px solid rgba(240,237,227,0.18)",
    color: "rgba(240,237,227,0.78)",
    borderRadius: "999px",
    fontFamily: "var(--font-mono, 'SF Mono', monospace)",
    fontSize: "0.66rem",
    letterSpacing: "0.3em",
    textTransform: "uppercase",
    cursor: "pointer",
    transition: "all 0.2s ease",
  },
  error: {
    margin: 0,
    fontFamily: "var(--font-mono, 'SF Mono', monospace)",
    fontSize: "0.7rem",
    letterSpacing: "0.18em",
    color: "#FF8A8A",
    textTransform: "uppercase",
  },
  footerHint: {
    position: "absolute",
    bottom: "clamp(1rem, 3vh, 2rem)",
    left: "50%",
    transform: "translateX(-50%)",
    margin: 0,
    fontFamily: "var(--font-mono, 'SF Mono', monospace)",
    fontSize: "0.68rem",
    letterSpacing: "0.22em",
    color: "rgba(240,237,227,0.45)",
    textTransform: "uppercase",
  },
  footerLink: {
    color: "#C5A017",
    textDecoration: "none",
    borderBottom: "1px solid rgba(197,160,23,0.3)",
  },
};
