import { defineConfig } from '@playwright/test';

// Serves the built Trunk output; all asset URLs are relative so any root works.
// Reuses the Chromium already in ~/.cache/ms-playwright unless overridden.
export default defineConfig({
  testDir: './tests',
  outputDir: './test-results',
  use: { baseURL: 'http://127.0.0.1:4173/', browserName: 'chromium' },
  webServer: {
    command: 'npx http-server ../crates/web/dist -p 4173 -a 127.0.0.1 -c-1 --silent',
    url: 'http://127.0.0.1:4173/',
    reuseExistingServer: true,
  },
});
