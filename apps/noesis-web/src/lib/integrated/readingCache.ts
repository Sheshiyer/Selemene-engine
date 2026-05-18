// ─── readingCache — anonymous localStorage persistence for readings ────
// The ephemeral-first contract:
//   - Anonymous users get FULL reading experience: generate → view → cache
//   - Last reading + history-of-10 stored in localStorage on this device
//   - No backend dependency for read-back: payload lives in the URL hash
//     too, so links are sharable + reload-safe
//   - Sign-in is optional, opt-in AFTER a reading exists. Clicking
//     "Save to your account" stashes a pendingClaim, fires OAuth, and
//     on return associates the reading with the user's account.
//
// Storage keys (all under the "noesis." namespace to avoid clashing):
//   - noesis.lastReading        — JSON of the most recent ReadingPayload
//   - noesis.readingHistory     — JSON array of last 10 cache entries
//   - noesis.pendingClaim       — reading_id awaiting post-auth claim
//   - witness-agents.daily-witness-form  — form auto-fill (pre-existing)

import type { ReadingPayload } from "./payloadLoader";

const KEY_LAST = "noesis.lastReading";
const KEY_HISTORY = "noesis.readingHistory";
const KEY_PENDING_CLAIM = "noesis.pendingClaim";

const HISTORY_CAP = 10;

export interface CachedReading {
  /** The reading payload itself. */
  payload: ReadingPayload;
  /** When this reading landed on this device. */
  cached_at: string; // ISO 8601
  /** Whether the user has bound this reading to their account
   *  (set after successful claim). */
  claimed?: boolean;
}

const isBrowser = (): boolean => typeof window !== "undefined";

const safeRead = <T>(key: string): T | null => {
  if (!isBrowser()) return null;
  try {
    const raw = window.localStorage.getItem(key);
    return raw ? (JSON.parse(raw) as T) : null;
  } catch {
    return null;
  }
};

const safeWrite = (key: string, value: unknown): void => {
  if (!isBrowser()) return;
  try {
    window.localStorage.setItem(key, JSON.stringify(value));
  } catch {
    // QuotaExceeded or private-browsing — silently degrade
  }
};

const safeRemove = (key: string): void => {
  if (!isBrowser()) return;
  try {
    window.localStorage.removeItem(key);
  } catch {
    /* ignore */
  }
};

// ─── Last reading (single-slot) ─────────────────────────────────────────

/** Save the most recent reading. Also appends to history. */
export function cacheReading(payload: ReadingPayload): CachedReading {
  const entry: CachedReading = {
    payload,
    cached_at: new Date().toISOString(),
    claimed: false,
  };
  safeWrite(KEY_LAST, entry);
  appendToHistory(entry);
  return entry;
}

/** Read the most recent cached reading, or null. */
export function getLastReading(): CachedReading | null {
  return safeRead<CachedReading>(KEY_LAST);
}

// ─── History (capped FIFO) ──────────────────────────────────────────────

function appendToHistory(entry: CachedReading): void {
  const existing = safeRead<CachedReading[]>(KEY_HISTORY) ?? [];
  // Dedupe by reading_id — if the same reading is regenerated/refreshed,
  // bump its cached_at and move it to the front, don't duplicate.
  const id = entry.payload.reading_id;
  const filtered = id
    ? existing.filter((e) => e.payload.reading_id !== id)
    : existing;
  const next = [entry, ...filtered].slice(0, HISTORY_CAP);
  safeWrite(KEY_HISTORY, next);
}

/** Return the device's reading history, newest first. */
export function getReadingHistory(): CachedReading[] {
  return safeRead<CachedReading[]>(KEY_HISTORY) ?? [];
}

/** Look up a reading from history by id. Returns null if not found. */
export function getCachedReadingById(id: string): CachedReading | null {
  if (!id) return null;
  const last = getLastReading();
  if (last?.payload.reading_id === id) return last;
  const history = getReadingHistory();
  return history.find((e) => e.payload.reading_id === id) ?? null;
}

/** Mark a reading as claimed (after successful POST to /readings/{id}/claim). */
export function markReadingClaimed(id: string): void {
  if (!id) return;
  const last = getLastReading();
  if (last?.payload.reading_id === id) {
    safeWrite(KEY_LAST, { ...last, claimed: true });
  }
  const history = getReadingHistory();
  const updated = history.map((e) =>
    e.payload.reading_id === id ? { ...e, claimed: true } : e,
  );
  safeWrite(KEY_HISTORY, updated);
}

/** Drop everything (used on explicit sign-out, never automatically). */
export function clearReadingCache(): void {
  safeRemove(KEY_LAST);
  safeRemove(KEY_HISTORY);
  safeRemove(KEY_PENDING_CLAIM);
}

// ─── Pending claim (for the OAuth roundtrip) ────────────────────────────

/** Stash the id of a reading the user wants to bind to their account
 *  after they finish OAuth. The post-callback handler reads this. */
export function setPendingClaim(readingId: string): void {
  if (!readingId) return;
  safeWrite(KEY_PENDING_CLAIM, { reading_id: readingId, queued_at: new Date().toISOString() });
}

export function getPendingClaim(): { reading_id: string; queued_at: string } | null {
  return safeRead<{ reading_id: string; queued_at: string }>(KEY_PENDING_CLAIM);
}

export function clearPendingClaim(): void {
  safeRemove(KEY_PENDING_CLAIM);
}
