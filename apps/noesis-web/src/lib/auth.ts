const API_KEY_STORAGE = "noesis_api_key";
const USER_PROFILE_STORAGE = "noesis_user_profile";

export interface UserProfile {
  id: string;
  email: string;
  full_name: string;
  tier: string;
  is_admin: boolean;
}

export function getApiKey(): string | null {
  if (typeof window === "undefined") return null;
  return localStorage.getItem(API_KEY_STORAGE);
}

export function setApiKey(key: string): void {
  localStorage.setItem(API_KEY_STORAGE, key);
}

export function clearApiKey(): void {
  localStorage.removeItem(API_KEY_STORAGE);
  localStorage.removeItem(USER_PROFILE_STORAGE);
}

export function isAuthenticated(): boolean {
  return getApiKey() !== null;
}

export function getUserProfile(): UserProfile | null {
  if (typeof window === "undefined") return null;
  const raw = localStorage.getItem(USER_PROFILE_STORAGE);
  if (!raw) return null;
  try { return JSON.parse(raw) as UserProfile; } catch { return null; }
}

export function setUserProfile(profile: UserProfile): void {
  localStorage.setItem(USER_PROFILE_STORAGE, JSON.stringify(profile));
}
