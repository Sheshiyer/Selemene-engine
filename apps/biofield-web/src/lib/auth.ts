export interface BiofieldAuthSession {
  token: string;
  userId: string;
  email: string;
  tier: string;
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
