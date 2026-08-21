import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    environment: 'jsdom',
    setupFiles: ['./src/test/setup.js'],
    globals: true,
    // pkg/ is wasm-pack's generated output (only present after `npm run
    // build:wasm`), not source — nothing there is meant to be imported by
    // tests, which mock the wasm module at the component boundary instead.
    exclude: ['**/node_modules/**', '**/pkg/**'],
  },
});
