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
import { getApiKey, isAuthenticated } from "@/lib/auth";

/** localStorage key holding `"1"` while the user is in the middle of a
 *  Discord-OAuth round-trip for the integrated reading. Set just before
 *  redirecting to /auth, read on /get-reading mount to auto-replay the
 *  submission once the user comes back authenticated. */
const PENDING_INTEGRATED_KEY = "noesis:pending_integrated";
import {
  DyadChamber,
  StepIndicator,
  SPEAKER_COLOR,
  SPEAKER_LABEL,
  type Speaker,
  type WitnessPose,
  type DyadStep,
} from "@selemene/dyad-ui";

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
 *  character lit + forward, the other dimmed + watching). The Speaker
 *  type and SPEAKER_LABEL constant now live in @/components/dyad. */
const STEP_SPEAKER: Record<Step, Speaker> = {
  0: "both",
  1: "aletheios",
  2: "pichet",
  3: "aletheios",
  4: "pichet",
  5: "both",
};

/** Pose choreography per step so the dyad shifts physically across
 *  the ritual instead of standing static. Asset constraint: Pichet has
 *  front/left/right native poses, Aletheios has front/right (no left).
 *
 *  Pattern: when a witness is the SPEAKER they face front (looking at
 *  the user). When they're listening, they turn slightly toward the
 *  speaker side. Pichet's left/right alternates so the motion across
 *  steps reads as alive rather than flipping a single switch. */
const STEP_POSES: Record<Step, { pichet: WitnessPose; aletheios: WitnessPose }> = {
  0: { pichet: "front", aletheios: "front" }, // both — joined field
  1: { pichet: "right", aletheios: "front" }, // Aletheios names; Pichet turns toward her
  2: { pichet: "front", aletheios: "right" }, // Pichet asks; Aletheios turns toward him (her only inward angle)
  3: { pichet: "left",  aletheios: "front" }, // Aletheios returns; Pichet shifts to other side (uses 3rd pose)
  4: { pichet: "front", aletheios: "right" }, // Pichet roots; Aletheios listens
  5: { pichet: "front", aletheios: "front" }, // both — depth fork
};

/** Sigil-token shapes per step for the StepIndicator. Steps 1+ visible;
 *  step 0 is the intro (no indicator shown). */
const FLOW_STEPS: ReadonlyArray<DyadStep> = [
  { speaker: "aletheios", symbol: "vesica" },
  { speaker: "pichet",    symbol: "hex" },
  { speaker: "aletheios", symbol: "vesica" },
  { speaker: "pichet",    symbol: "hex" },
  { speaker: "both",      symbol: "trinity" },
];

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

      /* Discord OAuth gate for the integrated (Pichet · Sixteen mirrors)
         path: the full-spectrum workflow is a heavyweight 17-engine
         compute meant to be tied to an identity and saved across
         sessions. If the user is anonymous, we round-trip them through
         Discord OAuth before running the workflow, then auto-replay on
         return. Daily-practice stays anonymous-first. */
      if (workflow === "integrated" && !isAuthenticated()) {
        try {
          localStorage.setItem(PENDING_INTEGRATED_KEY, "1");
        } catch { /* quota — degrade to ungated submit */ }
        // Persist form state happens via the existing useEffect on
        // state changes, so the round-trip preserves date/time/place/name.
        window.location.assign("/auth?next=/get-reading");
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
        const apiKey = workflow === "integrated" ? getApiKey() : null;
        const headers: Record<string, string> = { "Content-Type": "application/json" };
        if (apiKey) {
          // Match the convention used elsewhere (lib/api.ts): keys
          // prefixed with `nk_` use the x-api-key header; OAuth-derived
          // tokens (Discord login → JWT) use Authorization: Bearer.
          if (apiKey.startsWith("nk_")) {
            headers["x-api-key"] = apiKey;
          } else {
            headers["Authorization"] = `Bearer ${apiKey}`;
          }
        }
        const res = await fetch(url, {
          method: "POST",
          headers,
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

  /* OAuth round-trip replay: if the user was bounced through Discord
     login from the integrated path, the PENDING_INTEGRATED_KEY flag
     is sitting in localStorage. As soon as the hydrated form has the
     fields it needs (date + location) AND we're now authenticated,
     fire the integrated submit automatically — the user sees the
     workflow run without having to re-click. Guarded by a ref so it
     only fires once per mount. */
  const integratedReplayedRef = useRef(false);
  useEffect(() => {
    if (integratedReplayedRef.current) return;
    if (!state.birth_date || !state.location_key) return;
    let pending: string | null = null;
    try {
      pending = localStorage.getItem(PENDING_INTEGRATED_KEY);
    } catch { /* ignore */ }
    if (pending !== "1") return;
    if (!isAuthenticated()) return;
    // All conditions met — clear flag, fire submission.
    try {
      localStorage.removeItem(PENDING_INTEGRATED_KEY);
    } catch { /* ignore */ }
    integratedReplayedRef.current = true;
    submit("integrated");
  }, [state.birth_date, state.location_key, submit]);

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
      {/* Metallic fractalNoise grain over the fluid gradient. Pure
          atmosphere — pointer-events:none so it never blocks clicks. */}
      <div className="threshold-noise" aria-hidden="true" />

      <DyadChamber
        speaker={STEP_SPEAKER[state.step]}
        submitting={state.submitting !== null}
        pichetPose={STEP_POSES[state.step].pichet}
        aletheiosPose={STEP_POSES[state.step].aletheios}
      />

      <div style={s.stage} onClick={(e) => e.stopPropagation()}>
        <StepIndicator
          steps={FLOW_STEPS}
          currentIndex={state.step - 1}
          onJump={(i) => setStep((i + 1) as Step)}
          ariaLabel="Threshold ritual progress"
        />

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

      {/* Keyframes (thresholdPulse, thresholdRingDrift, thresholdStepFade)
          and the .threshold-step-enter helper now live in app/globals.css
          under the DYAD-CHAMBER section, since DyadChamber is canonical
          and used outside this file. */}
    </div>
  );
}

// DyadChamber, WitnessFigure, StepIndicator, and SigilToken
// were extracted to src/components/dyad/ — see the import at the
// top of this file. They are now the canonical guided-flow chrome
// for the entire app.

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

  // Compose YYYY-MM-DD when all three are valid.
  //
  // Guard against infinite render loop: the parent passes a fresh
  // `onChange` reference on every render, so without a value-equality
  // check this effect would fire → setState → parent re-renders →
  // new onChange identity → effect re-fires forever.
  // The `composed !== value` check makes the effect idempotent.
  useEffect(() => {
    if (/^\d{1,2}$/.test(d) && /^\d{1,2}$/.test(m) && /^\d{4}$/.test(y)) {
      const dd = d.padStart(2, "0");
      const mm = m.padStart(2, "0");
      if (parseInt(dd, 10) >= 1 && parseInt(dd, 10) <= 31 && parseInt(mm, 10) >= 1 && parseInt(mm, 10) <= 12) {
        const composed = `${y}-${mm}-${dd}`;
        if (composed !== value) onChange(composed);
      }
    }
  }, [d, m, y, value, onChange]);

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

  // Same idempotency guard as DateInput — compose only when the new
  // value differs from the current prop, else parent's fresh onChange
  // reference triggers an infinite render loop.
  useEffect(() => {
    if (/^\d{1,2}$/.test(h) && /^\d{1,2}$/.test(m)) {
      const hh = h.padStart(2, "0");
      const mm = m.padStart(2, "0");
      if (parseInt(hh, 10) <= 23 && parseInt(mm, 10) <= 59) {
        const composed = `${hh}:${mm}`;
        if (composed !== value) onChange(composed);
      }
    } else if (!h && !m && value !== "") {
      onChange("");
    }
  }, [h, m, value, onChange]);

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
    /* Metallic fluid backdrop — four overlapping radial gradients in
       the Goethe palette, drifting slowly via background-position over
       32s. Read together the gradients form a shifting metallic field
       rather than a single static vignette. The black floor (#070B1D)
       sits underneath everything so the gradients tint without losing
       depth. A fractalNoise overlay is applied in .threshold-noise
       below via globals.css for the metallic texture. */
    background: [
      "radial-gradient(ellipse 70% 55% at 18% 28%, rgba(45, 0, 80, 0.58), transparent 62%)",
      "radial-gradient(ellipse 60% 65% at 82% 72%, rgba(11, 80, 251, 0.36), transparent 62%)",
      "radial-gradient(ellipse 45% 40% at 62% 16%, rgba(197, 160, 23, 0.20), transparent 58%)",
      "radial-gradient(ellipse 45% 40% at 28% 84%, rgba(16, 181, 167, 0.20), transparent 58%)",
      "#070B1D",
    ].join(", "),
    backgroundSize: "220% 220%, 220% 220%, 220% 220%, 220% 220%, auto",
    animation: "thresholdFluidDrift 32s ease-in-out infinite alternate",
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
    // Was min(560px, 92vw) — too narrow on wide viewports, forced the
    // heading to wrap to 5 short lines. Widened to ~820px so the prompt
    // copy has room to breathe between the two witnesses.
    width: "min(820px, 92vw)",
    display: "flex",
    flexDirection: "column",
    alignItems: "center",
    gap: "clamp(1.25rem, 2.5vh, 2rem)",
    // Vignette behind the center column so the witness figures recede
    // visually and the dialog reads cleanly. Widened with the column.
    background:
      "radial-gradient(ellipse at center, rgba(7,11,29,0.62) 0%, rgba(7,11,29,0.0) 70%)",
    padding: "clamp(1rem, 3vh, 2.5rem) clamp(1rem, 4vw, 3rem)",
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
    maxWidth: "56ch",
    lineHeight: 1.6,
  },
  question: {
    margin: 0,
    fontFamily: "var(--font-display, 'Panchang', serif)",
    fontVariationSettings: "'wght' 720",
    // Capped at 2.5em (was 3em) so the heading fits 2-3 lines in the
    // ~770px channel between the two witnesses on wide viewports
    // instead of breaking to 4-5 lines. Still scales down at smaller
    // viewports via the clamp lower bound.
    fontSize: "clamp(1.6em, min(3.6vw, 6vh), 2.5em)",
    lineHeight: 1.08,
    letterSpacing: "-0.012em",
    color: "#F0EDE3",
    textAlign: "center" as const,
    // The h1 is a flex-column child under `alignItems: center`, which
    // makes it collapse to its content's natural width — so a bare
    // maxWidth never took effect. width:100% forces the h1 to fill
    // the available column, then maxWidth caps it at the stage width.
    width: "100%",
    // Tracks the stage column (820px). Lets the heading wrap on the
    // natural sentence break ("Coordinates required." / "The date that
    // anchored your arrival.") instead of word-by-word breaks.
    maxWidth: "min(780px, 100%)",
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
