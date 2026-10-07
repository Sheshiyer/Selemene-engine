import type { ValidatedEnv } from "../../selemene-mcp/src/env.js";

export const PUBLIC_ORIGIN = "https://selemene-free-mcp.sheshnarayan-iyer.workers.dev";
const UPSTREAMS = new Set(["https://selemene-engine-production.up.railway.app", "https://selemene.tryambakam.space"]);
export interface Env {
  PUBLIC_ORIGIN: string;
  SELEMENE_API_ORIGIN: string;
  SELEMENE_API_KEY: string;
  RATE_LIMIT_SALT: string;
  FREE_RATE_LIMITER: DurableObjectNamespace;
  IP_REQUESTS_PER_MINUTE?: string;
  IP_CALCULATIONS_PER_MINUTE?: string;
  GLOBAL_REQUESTS_PER_MINUTE?: string;
  GLOBAL_CALCULATIONS_PER_MINUTE?: string;
  GLOBAL_CALCULATIONS_PER_DAY?: string;
}
export interface Limits {
  ipRequests: number; ipCalculations: number; globalRequests: number;
  globalCalculations: number; dailyCalculations: number;
}
export function readLimits(env: Env): Limits {
  function limit(value: string | undefined, fallback: number, maximum: number): number {
    if (value === undefined) return fallback;
    if (!/^[1-9]\d{0,5}$/.test(value)) throw new Error("Invalid quota configuration.");
    const n = Number(value);
    if (n > maximum) throw new Error("Invalid quota configuration.");
    return n;
  }
  return {
    ipRequests: limit(env.IP_REQUESTS_PER_MINUTE, 120, 1000),
    ipCalculations: limit(env.IP_CALCULATIONS_PER_MINUTE, 30, 100),
    globalRequests: limit(env.GLOBAL_REQUESTS_PER_MINUTE, 600, 10000),
    globalCalculations: limit(env.GLOBAL_CALCULATIONS_PER_MINUTE, 60, 1000),
    dailyCalculations: limit(env.GLOBAL_CALCULATIONS_PER_DAY, 1000, 100000),
  };
}
export interface FreeContext {
  upstream: ValidatedEnv;
  apiKey: string;
  salt: string;
  limits: Limits;
  limiter: DurableObjectNamespace;
}
export function readEnv(env: Env): FreeContext {
  if (env.PUBLIC_ORIGIN !== PUBLIC_ORIGIN || !UPSTREAMS.has(env.SELEMENE_API_ORIGIN)) throw new Error("Invalid service origin.");
  if (typeof env.SELEMENE_API_KEY !== "string" || env.SELEMENE_API_KEY.length < 16 ||
      env.SELEMENE_API_KEY.length > 256 || /\s/.test(env.SELEMENE_API_KEY)) throw new Error("Sponsor credential is not configured.");
  if (typeof env.RATE_LIMIT_SALT !== "string" || env.RATE_LIMIT_SALT.length < 32 ||
      env.RATE_LIMIT_SALT.length > 256 || !env.FREE_RATE_LIMITER) throw new Error("Rate limiting is not configured.");
  return {
    upstream: { publicOrigin: PUBLIC_ORIGIN, mcpResource: `${PUBLIC_ORIGIN}/mcp`,
      authorizeEndpoint: "", tokenEndpoint: "", clientRegistrationEndpoint: "",
      selemeneApiOrigin: env.SELEMENE_API_ORIGIN },
    apiKey: env.SELEMENE_API_KEY, salt: env.RATE_LIMIT_SALT,
    limits: readLimits(env), limiter: env.FREE_RATE_LIMITER,
  };
}
