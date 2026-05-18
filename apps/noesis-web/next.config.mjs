import path from "node:path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

/** @type {import('next').NextConfig} */
const nextConfig = {
  output: "standalone",
  reactStrictMode: true,
  // ─── Monorepo workspace root (Turbopack-specific) ─────────────────────
  // This repo is an npm-workspaces monorepo with multiple package.json
  // files. Without explicitly setting `turbopack.root`, Next.js 16
  // Turbopack throws "inferred your workspace root, but it may not be
  // correct" during production build. We pin it to the repo root (two
  // levels up: apps/noesis-web → apps → repo root).
  //
  // We deliberately do NOT set `outputFileTracingRoot` here — doing so
  // moves the .next/ output above the app directory, which breaks the
  // Vercel CLI deploy that expects .next/ relative to the CWD.
  turbopack: {
    root: path.join(__dirname, "../../"),
  },
};

export default nextConfig;
