import { describe, expect, it } from "vitest";
import {
  BIOFIELD_CAPTURE_STATES,
  BIOFIELD_ENGINE_ID,
  BIOFIELD_SESSION_STATUSES,
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
});
