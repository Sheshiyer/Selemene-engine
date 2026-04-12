const STORAGE_KEY = "selemene_biofield_active_session_id";
const listeners = new Set<() => void>();

function emitChange(): void {
  listeners.forEach((listener) => listener());
}

export function getStoredActiveSessionId(): string | null {
  if (typeof window === "undefined") {
    return null;
  }

  const raw = window.localStorage.getItem(STORAGE_KEY);
  if (!raw) {
    return null;
  }

  const sessionId = raw.trim();
  if (!sessionId) {
    window.localStorage.removeItem(STORAGE_KEY);
    return null;
  }

  return sessionId;
}

export function subscribeToActiveSessionId(listener: () => void): () => void {
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

export function setStoredActiveSessionId(sessionId: string): void {
  if (typeof window === "undefined") {
    return;
  }

  const normalized = sessionId.trim();
  if (!normalized) {
    window.localStorage.removeItem(STORAGE_KEY);
  } else {
    window.localStorage.setItem(STORAGE_KEY, normalized);
  }

  emitChange();
}

export function clearStoredActiveSessionId(): void {
  if (typeof window === "undefined") {
    return;
  }

  window.localStorage.removeItem(STORAGE_KEY);
  emitChange();
}
