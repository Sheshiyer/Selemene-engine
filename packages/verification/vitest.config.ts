import { defineConfig } from 'vitest/config';

// NOTE: The "Cannot find base config file astro/tsconfigs/strict" warning comes
// from a parent-directory tsconfig.json outside this repo and is harmless.

export default defineConfig({
  test: {
    globals: true,
    environment: 'node',
    include: ['src/**/*.test.ts', 'tests/**/*.test.ts'],
    passWithNoTests: true,
    env: {
      SELEMENE_API_URL: process.env.SELEMENE_API_URL ?? 'https://selemene.tryambakam.space',
    },
  },
});
