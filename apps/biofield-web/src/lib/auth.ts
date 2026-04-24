export interface BiofieldAuthSession {
  token: string;
  userId: string;
  email: string;
  tier: string;
}

const STORAGE_KEY = "selemene_biofield_auth";
const listeners = new Set<() => void>();

function emitChange(): void {
  listeners.forEach((listener) => listener());
}

export function getStoredAuthSession(): BiofieldAuthSession | null {
  if (typeof window === "undefined") {
    return null;
  }

  const raw = window.localStorage.getItem(STORAGE_KEY);
  if (!raw) {
    return null;
  }

  try {
    return JSON.parse(raw) as BiofieldAuthSession;
  } catch {
    window.localStorage.removeItem(STORAGE_KEY);
    return null;
  }
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
