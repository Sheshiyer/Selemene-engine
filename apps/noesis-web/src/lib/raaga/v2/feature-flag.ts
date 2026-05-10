// V2 feature flag. Drives whether RaagaPlayer activates the v2 path
// (gamakas + samples + tala + breath + offline render) or stays on the
// v1 sine-wave just-intonation path.
//
// Default: OFF. Flip via env or programmatic override.

const ENV_KEY = 'RAAGA_V2_ENABLED';

export const isV2Enabled = (): boolean => {
  // Server-side / build-time: read env. Treat any truthy non-"false" as on.
  if (typeof process !== 'undefined' && process.env) {
    const v = process.env[ENV_KEY] ?? process.env[`NEXT_PUBLIC_${ENV_KEY}`];
    if (v == null) return false;
    return v !== 'false' && v !== '0' && v !== '';
  }
  // Client-side fallback: localStorage override for QA.
  if (typeof window !== 'undefined' && window.localStorage) {
    return window.localStorage.getItem(ENV_KEY) === 'true';
  }
  return false;
};

/** Programmatic override for tests. Resets on page reload. */
let _override: boolean | null = null;
export const setV2EnabledOverride = (v: boolean | null): void => { _override = v; };
export const v2Enabled = (): boolean => _override ?? isV2Enabled();
