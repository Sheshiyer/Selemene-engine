// Reviewed adapter input schemas per engine.
//
// STRICT per-engine option keys. No arbitrary maps. Same validation applies
// to prepare_reading and calculate. Prepare accepts incomplete birth_data
// and reports missing fields. Nadabrahman and sacred-geometry are removed:
// their only evidence is "question-only claims" without verification that
// the engine operates correctly without birth_data or location.

import { z } from "zod";
import type { AllowedEngineId } from "./allowlist.js";
export type { AllowedEngineId } from "./allowlist.js";

// ---- Shared primitives -----------------------------------------------------

const DateStr = z
  .string()
  .regex(/^\d{4}-\d{2}-\d{2}$/, "date must be YYYY-MM-DD")
  .refine((s) => {
    const [y, m, d] = s.split("-").map(Number);
    if (y < 1000 || y > 3000) return false;
    const dt = new Date(y, m - 1, d);
    return (
      dt.getFullYear() === y &&
      dt.getMonth() === m - 1 &&
      dt.getDate() === d
    );
  }, "must be a real calendar date (year 1000–3000)");

const TimeStr = z
  .string()
  .regex(
    /^([01]\d|2[0-3]):[0-5]\d$/,
    "time must be HH:MM (24-hour clock)",
  );

const IanaTimezone = z
  .string()
  .min(1, "timezone required")
  .refine(
    (s) => { try { new Intl.DateTimeFormat("en", { timeZone: s }); return true; } catch { return false; } },
    "must be an IANA timezone (e.g. Europe/Paris, America/New_York)",
  );

const Latitude = z.number().finite().min(-90).max(90);
const Longitude = z.number().finite().min(-180).max(180);


const BirthDataFull = z
  .object({
    name: z.string().trim().min(1).optional(),
    date: DateStr,
    time: TimeStr,
    latitude: Latitude,
    longitude: Longitude,
    timezone: IanaTimezone,
  })
  .strict();

const BirthDataDateOnly = BirthDataFull.extend({
  name: z.string().trim().min(1).optional(),
  time: TimeStr.nullable().optional(),
}).strict();

// ---- Per-engine schemas ----------------------------------------------------

const NumerologySchema = z
  .object({
    engine_id: z.literal("numerology"),
    birth_data: BirthDataDateOnly.extend({ name: z.string().trim().min(1) }).describe(
      "REQUIRED. name is the full birth name. Parameters.full_name is NOT accepted.",
    ),
    parameters: z.object({}).strict().optional(),
  })
  .strict();

const HumanDesignSchema = z
  .object({
    engine_id: z.literal("human-design"),
    birth_data: BirthDataFull.describe(
      "REQUIRED. Exact time of birth is mandatory for Human Design charts.",
    ),
    parameters: z.object({}).strict().optional(),
  })
  .strict();

const GeneKeysSchema = z
  .object({
    engine_id: z.literal("gene-keys"),
    birth_data: BirthDataFull.describe(
      "REQUIRED. Exact time of birth is mandatory for Gene Keys.",
    ),
    parameters: z.object({}).strict().optional(),
  })
  .strict();

const VimshottariSchema = z
  .object({
    engine_id: z.literal("vimshottari"),
    birth_data: BirthDataFull.describe(
      "REQUIRED. Exact time of birth is mandatory for Vimshottari dasha.",
    ),
    parameters: z.object({}).strict().optional(),
  })
  .strict();

const PanchangaSchema = z
  .object({
    engine_id: z.literal("panchanga"),
    current_time: z
      .string()
      .datetime({ offset: true })
      .describe("REQUIRED. RFC 3339 timestamp for the panchanga calculation."),
    birth_data: BirthDataFull.describe(
      "REQUIRED for natal panchanga. Full birth_data with exact time. " +
        "Parameters map must be empty. Current_time-only does NOT work without mode=daily.",
    ),
    parameters: z.object({}).strict().optional(),
  })
  .strict();

const VedicClockSchema = z
  .object({
    engine_id: z.literal("vedic-clock"),
    current_time: z.string().datetime({ offset: true }),
    birth_data: z
      .object({
        name: z.string().min(1),
        date: DateStr,
        latitude: Latitude,
        longitude: Longitude,
        timezone: IanaTimezone.describe(
          "Explicit IANA timezone required. No unknown UTC defaults.",
        ),
      })
      .strict()
      .optional(),
    parameters: z
      .object({
        timezone_offset: z
          .number()
          .int()
          .min(-780)
          .max(780)
          .optional()
          .describe(
            "Explicit UTC offset in minutes. Required if birth_data.timezone is absent.",
          ),
      })
      .strict()
      .optional(),
  })
  .strict()
  .refine(
    (data) =>
      data.birth_data?.timezone != null ||
      data.parameters?.timezone_offset != null,
    "Either birth_data.timezone or parameters.timezone_offset is required",
  );

const BiorhythmSchema = z
  .object({
    engine_id: z.literal("biorhythm"),
    birth_data: BirthDataDateOnly.describe(
      "REQUIRED. Birth date and location for biorhythm calculation.",
    ),
    current_time: z
      .string()
      .datetime({ offset: true })
      .describe("REQUIRED. Explicit current time for biorhythm curves."),
    parameters: z.object({}).strict().optional(),
  })
  .strict();

const TransitsSchema = z
  .object({
    engine_id: z.literal("transits"),
    birth_data: BirthDataFull.describe(
      "REQUIRED. Exact birth time is mandatory for transits against natal chart.",
    ),
    current_time: z
      .string()
      .datetime({ offset: true })
      .optional()
      .describe("Defaults to now if not provided."),
    parameters: z.object({}).strict().optional(),
  })
  .strict();

const EnneagramAnswers = z
  .array(z.number().int().min(1).max(5))
  .length(45, "answers must be exactly 45 integers (1–5)");

const EnneagramSchema = z
  .object({
    engine_id: z.literal("enneagram"),
    options: z
      .object({
        answers: EnneagramAnswers.optional().describe(
          "45 integer answers (1–5) from the Riso-Hudson enneagram inventory.",
        ),
        type: z
          .number()
          .int()
          .min(1)
          .max(9)
          .optional()
          .describe("Known enneagram type (1–9) if already identified."),
        wing: z
          .number()
          .int()
          .min(1)
          .max(9)
          .optional()
          .describe("Adjacent wing (1–9)."),
        includeAssessment: z
          .boolean()
          .optional()
          .describe("Include full assessment breakdown in output."),
        includeMovementPrompts: z
          .boolean()
          .optional()
          .describe("Include integration/disintegration movement prompts."),
      })
      .strict()
      .refine(
        (o) => o.answers != null || o.type != null,
        "Either answers (45 integers 1–5) or type (1–9) is required; never question-based typing.",
      ).refine(o => o.wing === undefined || (o.type !== undefined && (o.wing === (o.type % 9) + 1 || o.wing === ((o.type + 7) % 9) + 1)), "Wing must be adjacent to supplied type."),
    parameters: z.object({}).strict().optional(),
  })
  .strict();

const TarotSchema = z
  .object({
    engine_id: z.literal("tarot"),
    options: z
      .object({
        spread: z
          .enum(["three_card", "celtic_cross", "single_card"])
          .optional()
          .describe("Spread layout. Defaults to single-card."),
        question: z
          .string()
          .min(1)
          .max(2000)
          .optional()
          .describe("The tarot inquiry."),
      })
      .strict(),
    parameters: z.object({}).strict().optional(),
  })
  .strict();

const IChingSchema = z
  .object({
    engine_id: z.literal("i-ching"),
    options: z
      .object({
        method: z
          .enum(["three_coins"])
          .optional()
          .describe("Divination method. Defaults to coins."),
        question: z
          .string()
          .min(1)
          .max(2000)
          .optional()
          .describe("The I Ching inquiry."),
      })
      .strict(),
    parameters: z.object({}).strict().optional(),
  })
  .strict();

// ---- Schema registry (maps AllowedEngineId → Zod schema) ------------------

type EngineSchemaEntry = {
  zodSchema: z.ZodType;
  title: string;
  summary: string;
  requires_birth_data: boolean;
  requires_exact_time: boolean;
  requires_current_time: boolean;
  side_effects: string[];
};

export const CALCULATION_SIDE_EFFECTS: readonly string[] = [
  "persists a reading row bound to the authenticated user",
  "logs a usage event and may increment the free-tier monthly counter",
  "awards experience points and may auto-promote the consciousness level",
  "may populate the user's stored profile from birth_data on first use",
];

const SE = [...CALCULATION_SIDE_EFFECTS];

export const ENGINE_SCHEMAS: Record<AllowedEngineId, EngineSchemaEntry> = {
  numerology: {
    zodSchema: NumerologySchema,
    title: "Numerology",
    summary: "Numerological analysis derived from name and date of birth.",
    requires_birth_data: true,
    requires_exact_time: false,
    requires_current_time: false,
    side_effects: SE,
  },
  "human-design": {
    zodSchema: HumanDesignSchema,
    title: "Human Design",
    summary: "Human Design chart requiring exact birth date, time, and location.",
    requires_birth_data: true,
    requires_exact_time: true,
    requires_current_time: false,
    side_effects: SE,
  },
  "gene-keys": {
    zodSchema: GeneKeysSchema,
    title: "Gene Keys",
    summary: "Gene Keys profile composed over Human Design; requires birth data with exact time.",
    requires_birth_data: true,
    requires_exact_time: true,
    requires_current_time: false,
    side_effects: SE,
  },
  vimshottari: {
    zodSchema: VimshottariSchema,
    title: "Vimshottari Dasha",
    summary: "Vedic dasha periods derived from a natal chart. Requires birth data with exact time.",
    requires_birth_data: true,
    requires_exact_time: true,
    requires_current_time: false,
    side_effects: SE,
  },
  panchanga: {
    zodSchema: PanchangaSchema,
    title: "Panchanga",
    summary: "Vedic almanac. Natal mode requires full birth_data with exact time and empty parameters.",
    requires_birth_data: true,
    requires_exact_time: true,
    requires_current_time: true,
    side_effects: SE,
  },
  "vedic-clock": {
    zodSchema: VedicClockSchema,
    title: "Vedic Clock",
    summary: "Time-of-day energetic mapping. Requires explicit timezone (IANA or offset minutes).",
    requires_birth_data: false,
    requires_exact_time: false,
    requires_current_time: true,
    side_effects: SE,
  },
  biorhythm: {
    zodSchema: BiorhythmSchema,
    title: "Biorhythm",
    summary: "Classical biorhythm curves. Requires birth data and explicit current_time.",
    requires_birth_data: true,
    requires_exact_time: false,
    requires_current_time: true,
    side_effects: SE,
  },
  transits: {
    zodSchema: TransitsSchema,
    title: "Transits",
    summary: "Current planetary transits against natal chart. Requires birth data with exact time.",
    requires_birth_data: true,
    requires_exact_time: true,
    requires_current_time: false,
    side_effects: SE,
  },
  enneagram: {
    zodSchema: EnneagramSchema,
    title: "Enneagram",
    summary: "Enneagram type via 45-answer inventory (1–5) or known type (1–9). Never question-based.",
    requires_birth_data: false,
    requires_exact_time: false,
    requires_current_time: false,
    side_effects: SE,
  },
  tarot: {
    zodSchema: TarotSchema,
    title: "Tarot",
    summary: "Symbolic tarot reflection with spread options.",
    requires_birth_data: false,
    requires_exact_time: false,
    requires_current_time: false,
    side_effects: SE,
  },
  "i-ching": {
    zodSchema: IChingSchema,
    title: "I Ching",
    summary: "I Ching hexagram reflection with divination method options.",
    requires_birth_data: false,
    requires_exact_time: false,
    requires_current_time: false,
    side_effects: SE,
  },
};

// Nadabrahman and sacred-geometry removed: insufficient verification of
// question-only operation without birth_data or location. Claims of
// "question-only" are unverified against the actual engine contract.

// ---- Validation helpers ----------------------------------------------------

export interface ValidationResult {
  valid: boolean;
  errors: string[];
  parsed?: Record<string, unknown>;
}

/**
 * Validate input for a given engine. Used by both prepare_reading (to report
 * what's missing) and calculate (to reject before upstream call).
 */
export function validateEngineInput(
  engineId: AllowedEngineId,
  raw: Record<string, unknown>,
  opts?: { partial?: boolean },
): ValidationResult {
  const schema = ENGINE_SCHEMAS[engineId];
  if (!schema) {
    return { valid: false, errors: [`unknown engine: ${engineId}`] };
  }

  if (opts?.partial) {
    // For prepare_reading: validate what's present, report what's missing.
    return validatePartial(engineId, raw, schema);
  }

  const result = schema.zodSchema.safeParse(raw);
  if (result.success) {
    return { valid: true, errors: [], parsed: result.data as Record<string, unknown> };
  }
  const errors = result.error.issues.map(
    (i) => `${i.path.filter((v) => typeof v === "number" || /^[a-z_]{1,32}$/i.test(String(v))).join(".")}: invalid input`,
  );
  return { valid: false, errors };
}

function validatePartial(
  _engineId: AllowedEngineId, raw: Record<string, unknown>, schema: EngineSchemaEntry,
): ValidationResult {
  const checked = schema.zodSchema.safeParse(raw);
  if (checked.success) return { valid: true, errors: [], parsed: { ...checked.data, _prepare_missing: [], _prepare_can_calculate: true } };
  const missing: string[] = [];
  const errors: string[] = [];
  for (const issue of checked.error.issues) {
    const path = issue.path.map((part) => typeof part === "number" ? "item" : part).join(".");
    if (issue.code === "invalid_type" && issue.received === "undefined") missing.push(path);
    else errors.push(path + ": invalid input");
  }
  return { valid: false, errors, parsed: { _prepare_missing: missing, _prepare_can_calculate: false } };
}

/**
 * For McpServer tool registration: convert an engine Zod schema to an
 * inputSchema JSON object. We use the raw Zod schema directly with
 * McpServer.tool().
 */
export function getEngineZodSchema(engineId: AllowedEngineId): z.ZodType | undefined {
  return ENGINE_SCHEMAS[engineId]?.zodSchema;
}
