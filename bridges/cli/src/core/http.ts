const DEFAULT_TIMEOUT = 5000;

export interface FetchOptions {
  timeout?: number;
  apiKey?: string;
  bearerToken?: string;
}

function authHeaders(opts: FetchOptions): Record<string, string> {
  if (opts.apiKey && opts.bearerToken) {
    throw new Error("ambiguous authentication: provide apiKey or bearerToken");
  }
  if (opts.apiKey) return { "X-API-Key": opts.apiKey };
  if (opts.bearerToken) return { Authorization: `Bearer ${opts.bearerToken}` };
  return {};
}

function safeErrorCode(payload: unknown): string {
  if (payload && typeof payload === "object" && "error_code" in payload) {
    const code = (payload as { error_code?: unknown }).error_code;
    if (typeof code === "string" && /^[A-Z0-9_]{1,64}$/.test(code)) return code;
  }
  return "HTTP_ERROR";
}

export async function fetchJSON<T = unknown>(
  url: string,
  opts: FetchOptions = {}
): Promise<T> {
  const { timeout = DEFAULT_TIMEOUT } = opts;

  const headers: Record<string, string> = {
    Accept: "application/json",
    ...authHeaders(opts),
  };

  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeout);

  try {
    const response = await fetch(url, {
      headers,
      signal: controller.signal,
    });

    if (!response.ok) {
      let payload: unknown = null;
      try {
        const text = await response.text();
        payload = text ? JSON.parse(text) : null;
      } catch {
        payload = null;
      }
      throw new Error(`${safeErrorCode(payload)} (${response.status})`);
    }

    const text = await response.text();
    if (!text) throw new Error("EMPTY_RESPONSE");
    try {
      return JSON.parse(text) as T;
    } catch {
      throw new Error("INVALID_JSON_RESPONSE");
    }
  } catch (err) {
    if (err instanceof Error && /^(HTTP_ERROR|[A-Z0-9_]+ \(\d+\)|EMPTY_RESPONSE|INVALID_JSON_RESPONSE)/.test(err.message)) {
      throw err;
    }
    throw new Error("NETWORK_ERROR");
  } finally {
    clearTimeout(timer);
  }
}

export async function checkHealth(
  url: string,
  opts: FetchOptions = {}
): Promise<{ ok: boolean; status?: number; error?: string }> {
  const headers = { Accept: "application/json", ...authHeaders(opts) };
  try {
    const controller = new AbortController();
    const timer = setTimeout(
      () => controller.abort(),
      opts.timeout ?? DEFAULT_TIMEOUT
    );

    const response = await fetch(url, {
      headers,
      signal: controller.signal,
    });

    clearTimeout(timer);
    return { ok: response.ok, status: response.status };
  } catch (err) {
    return {
      ok: false,
      error:
        err instanceof DOMException && err.name === "AbortError"
          ? "timeout"
          : "request_failed",
    };
  }
}
