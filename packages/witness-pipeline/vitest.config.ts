import { defineConfig } from 'vitest/config';
import { resolve } from 'path';

export default defineConfig({
  test: {
    globals: false,
    environment: 'node',
  },
  resolve: {
    alias: {
      '@noesis/witness-pipeline': resolve(__dirname, './src/index.ts'),
    },
  },
});
