// Sample loader — wraps Strudel's `samples()` with multi-CDN failover.
//
// Strategy:
//   1. Try the primary manifest URL (default: same-origin /raaga/v2/...).
//   2. On failure, walk the FALLBACK_CDNS list — jsdelivr, then unpkg-style
//      raw GitHub mirror.
//   3. Surface a structured error with attempted URLs so the UI can show
//      a useful message ("samples unavailable; check network").
//
// Idempotent: once any URL succeeds, subsequent calls are no-ops.

type StrudelEvaluate = (code: string) => Promise<unknown>;

let _loaded = false;
let _loading: Promise<void> | null = null;
let _activeUrl: string | null = null;

const DEFAULT_MANIFEST_URL = '/raaga/v2/samples-manifest.json';

/** Ordered fallbacks. First entry that resolves wins. */
export const FALLBACK_CDNS: readonly string[] = [
  'https://cdn.jsdelivr.net/gh/witnessOS/raaga-samples@main/manifest.json',
  'https://raw.githubusercontent.com/witnessOS/raaga-samples/main/manifest.json',
];

const fetchHeadOk = async (url: string): Promise<boolean> => {
  try {
    const resp = await fetch(url, { method: 'HEAD' });
    return resp.ok;
  } catch {
    return false;
  }
};

/**
 * Load the Nādashakti sample manifest. Tries `manifestUrl` first, falls back
 * through `FALLBACK_CDNS`. Returns the URL that succeeded.
 */
export const loadRaagaSamples = async (
  evaluate: StrudelEvaluate,
  manifestUrl: string = DEFAULT_MANIFEST_URL,
): Promise<string> => {
  if (_loaded && _activeUrl) return _activeUrl;
  if (_loading) {
    await _loading;
    if (_activeUrl) return _activeUrl;
  }

  _loading = (async () => {
    const candidates = [manifestUrl, ...FALLBACK_CDNS];
    const errors: string[] = [];

    for (const url of candidates) {
      // Quick reachability probe before asking Strudel to load it.
      // Strudel itself swallows fetch errors silently, which makes diagnosis hard.
      const reachable = await fetchHeadOk(url);
      if (!reachable) {
        errors.push(`${url} → not reachable`);
        continue;
      }
      try {
        await evaluate(`samples("${url}")`);
        _loaded = true;
        _activeUrl = url;
        return;
      } catch (err) {
        errors.push(`${url} → ${(err as Error).message}`);
      }
    }

    _loading = null;
    throw new Error(
      `loadRaagaSamples: all CDN candidates failed:\n  ${errors.join('\n  ')}`
    );
  })();

  await _loading;
  if (!_activeUrl) throw new Error('loadRaagaSamples: no active URL after load');
  return _activeUrl;
};

/** Test-only: reset internal state so tests can simulate fresh loads. */
export const resetSamplesForTests = (): void => {
  _loaded = false;
  _loading = null;
  _activeUrl = null;
};

/** Return the URL that successfully loaded, or null if not yet loaded. */
export const activeManifestUrl = (): string | null => _activeUrl;
