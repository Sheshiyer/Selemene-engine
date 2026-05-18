import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  // ─── Monorepo workspace root ─────────────────────────────────────────
  // This repo is an npm-workspaces monorepo with multiple package.json
  // files. Without explicitly setting outputFileTracingRoot, Next.js 16
  // Turbopack throws "inferred your workspace root, but it may not be
  // correct" during production build. Pin to the repo root (two levels
  // up: apps/noesis-web → apps → repo root).
  //
  // We removed `output: "standalone"` because (a) Vercel deploys via
  // its own runtime and doesn't need standalone output, and (b) the
  // combination of standalone + outputFileTracingRoot caused the
  // Vercel CLI deploy step to compute .next path as
  // ${CWD}/apps/noesis-web/.next/... (doubled path) and fail.
  outputFileTracingRoot: path.join(__dirname, "../../"),
};

export default nextConfig;
