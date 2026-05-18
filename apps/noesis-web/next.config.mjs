import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

/** @type {import('next').NextConfig} */
const nextConfig = {
  output: "standalone",
  reactStrictMode: true,
  // ─── Monorepo workspace root ─────────────────────────────────────────
  // This repo is an npm-workspaces monorepo with multiple package.json
  // files. Without this explicit setting, Next.js 16 + Turbopack throws
  // "inferred your workspace root, but it may not be correct" during
  // production build. Pin it to the repo root (two levels up from this
  // app: apps/noesis-web → apps → repo root).
  outputFileTracingRoot: path.join(__dirname, "../../"),
};

export default nextConfig;
