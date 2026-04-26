const DEFAULT_TIMEOUT = 5000;

interface FetchOptions {
  timeout?: number;
  apiKey?: string;
}

export async function fetchJSON<T = unknown>(
  url: string,
  opts: FetchOptions = {}
): Promise<T> {
  const { timeout = DEFAULT_TIMEOUT, apiKey } = opts;

  const headers: Record<string, string> = {
    Accept: "application/json",
  };
  if (apiKey) {
    headers["Authorization"] = `Bearer ${apiKey}`;
  }

  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeout);

  try {
    const response = await fetch(url, {
      headers,
      signal: controller.signal,
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    return (await response.json()) as T;
  } finally {
    clearTimeout(timer);
  }
}

export async function checkHealth(
  url: string,
  opts: FetchOptions = {}
): Promise<{ ok: boolean; status?: number; error?: string }> {
  try {
    const controller = new AbortController();
    const timer = setTimeout(
      () => controller.abort(),
      opts.timeout ?? DEFAULT_TIMEOUT
    );

    const response = await fetch(url, {
      signal: controller.signal,
    });

    clearTimeout(timer);
    return { ok: response.ok, status: response.status };
  } catch (err) {
    return {
      ok: false,
      error: err instanceof Error ? err.message : String(err),
    };
  }
}
