import { defineConfig } from 'vitest/config';
export default defineConfig({ resolve:{alias:{'cloudflare:workers': new URL('./test/cloudflare.js',import.meta.url).pathname}}, test:{server:{deps:{inline:['@cloudflare/workers-oauth-provider']}},include:['test/**/*.test.js','test/**/*.test.ts'],testTimeout:15000}});
