import { describe, expect, it } from "vitest";
import {
  BIOFIELD_CAPTURE_STATES,
  BIOFIELD_ENGINE_ID,
  BIOFIELD_SESSION_STATUSES,
  type BiofieldBaselineComparison,
  type BiofieldBaselineSummary,
  type BiofieldExportResult,
  type BiofieldReadingSummary,
} from "./index.js";

describe("biofield-domain", () => {
  it("freezes the native reading engine id", () => {
    expect(BIOFIELD_ENGINE_ID).toBe("biofield-capture");
  });

  it("defines the expected session and capture states", () => {
    expect(BIOFIELD_SESSION_STATUSES).toEqual(["active", "closed", "abandoned"]);
    expect(BIOFIELD_CAPTURE_STATES).toContain("persisted");
    expect(BIOFIELD_CAPTURE_STATES).toContain("rejected");
  });

  it("keeps reading summaries aligned to the frozen contract shape", () => {
    const summary: BiofieldReadingSummary = {
      reading_id: "reading-1",
      session_id: "session-1",
      engine_id: BIOFIELD_ENGINE_ID,
      created_at: "2026-04-05T12:05:00Z",
      quality: {
        sufficient_quality: true,
      },
      artifact: {
        kind: "source-image",
        mime_type: "image/jpeg",
      },
    };

    expect(summary.engine_id).toBe(BIOFIELD_ENGINE_ID);
    expect(summary.quality.sufficient_quality).toBe(true);
  });

  it("defines a minimal baseline summary shape", () => {
    const baseline: BiofieldBaselineSummary = {
      baseline_id: "baseline-1",
      name: "Morning baseline",
      notes: "Created from two accepted captures",
      reading_count: 2,
      created_at: "2026-04-09T12:05:00Z",
      updated_at: "2026-04-09T12:06:00Z",
    };

    expect(baseline.reading_count).toBe(2);
    expect(baseline.name).toContain("Morning");
  });

  it("defines comparison and export shapes for the deferred slice", () => {
    const comparison: BiofieldBaselineComparison = {
      comparison_version: "biofield-baseline-delta/v1",
      baseline: {
        baseline_id: "baseline-1",
        name: "Morning baseline",
        reading_count: 2,
        created_at: "2026-04-09T12:05:00Z",
        updated_at: "2026-04-09T12:06:00Z",
      },
      deltas: [
        {
          key: "light_quanta_density",
          reading_value: 42.5,
          baseline_value: 40.0,
          absolute_delta: 2.5,
          relative_delta: 0.0625,
        },
      ],
    };

    const exportResult: BiofieldExportResult = {
      export_id: "export-1",
      reading_id: "reading-1",
      baseline_id: "baseline-1",
      format: "json",
      file_name: "biofield-reading-reading-1.json",
      mime_type: "application/json",
      byte_size: 512,
      created_at: "2026-04-09T12:07:00Z",
      storage_path: "biofield/session-1/exports/export-1.json",
      bundle: {
        contract_version: "biofield-export/v1",
        reading: {
          comparison,
        },
      },
    };

    expect(comparison.deltas[0].absolute_delta).toBe(2.5);
    expect(exportResult.format).toBe("json");
    expect(exportResult.bundle.contract_version).toBe("biofield-export/v1");
  });
});
