import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    globals: true,
    environment: 'node',
    include: ['tests/**/*.test.ts'],
    env: {
      SELEMENE_API_URL: process.env.SELEMENE_API_URL ?? 'https://selemene.tryambakam.space',
    },
  },
});
