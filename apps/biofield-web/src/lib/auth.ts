export interface BiofieldAuthSession {
  token: string;
  userId: string;
  email: string;
  tier: string;
  /**
   * Effective admin permissions, populated post-login by GET /api/v1/admin/session.
   * Undefined until that fetch resolves; absent or empty for non-admin users.
   * Consumers MUST treat undefined and [] differently:
   *   undefined → still resolving (render skeleton)
   *   []        → resolved, no admin access (hide admin UI)
   */
  permissions?: string[];
}

const STORAGE_KEY = "selemene_biofield_auth";
const listeners = new Set<() => void>();

// Cached snapshot — same reference returned until storage actually changes.
// Required by useSyncExternalStore: getSnapshot must be referentially stable
// between renders when data has not changed.
let cachedSession: BiofieldAuthSession | null | undefined = undefined;

function readFromStorage(): BiofieldAuthSession | null {
  if (typeof window === "undefined") return null;
  const raw = window.localStorage.getItem(STORAGE_KEY);
  if (!raw) return null;
  try {
    return JSON.parse(raw) as BiofieldAuthSession;
  } catch {
    window.localStorage.removeItem(STORAGE_KEY);
    return null;
  }
}

function emitChange(): void {
  cachedSession = readFromStorage(); // refresh cache on every write/clear
  listeners.forEach((listener) => listener());
}

export function getStoredAuthSession(): BiofieldAuthSession | null {
  if (typeof window === "undefined") return null;
  // Populate cache on first call (hydration)
  if (cachedSession === undefined) {
    cachedSession = readFromStorage();
  }
  return cachedSession;
}

export function subscribeToAuthSession(listener: () => void): () => void {
  if (typeof window === "undefined") {
    return () => {};
  }

  listeners.add(listener);
  const handleStorage = () => listener();
  window.addEventListener("storage", handleStorage);

  return () => {
    listeners.delete(listener);
    window.removeEventListener("storage", handleStorage);
  };
}

export function setStoredAuthSession(session: BiofieldAuthSession): void {
  if (typeof window === "undefined") {
    return;
  }
  window.localStorage.setItem(STORAGE_KEY, JSON.stringify(session));
  emitChange();
}

export function clearStoredAuthSession(): void {
  if (typeof window === "undefined") {
    return;
  }
  window.localStorage.removeItem(STORAGE_KEY);
  emitChange();
}

/**
 * Update the stored session's permissions in place (or set to []) without
 * touching token/userId/email/tier. No-op if no session is stored.
 */
export function setStoredAuthPermissions(permissions: string[]): void {
  if (typeof window === "undefined") return;
  const current = readFromStorage();
  if (!current) return;
  const next: BiofieldAuthSession = { ...current, permissions };
  window.localStorage.setItem(STORAGE_KEY, JSON.stringify(next));
  emitChange();
}

/**
 * Returns true iff the session has any of the listed permissions, or the
 * `admin:*` wildcard. Returns false if permissions is undefined or empty.
 */
export function sessionHasAnyPermission(
  session: BiofieldAuthSession | null,
  required: string[],
): boolean {
  if (!session?.permissions || session.permissions.length === 0) return false;
  if (session.permissions.includes("admin:*")) return true;
  return required.some((perm) => session.permissions!.includes(perm));
}
