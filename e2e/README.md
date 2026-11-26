# E2E tests (Playwright)

```sh
cd crates/web && trunk build          # builds ./dist
cd e2e && npm ci                      # or `npm i` the first time
npx playwright test                   # serves ../crates/web/dist on :4173
```

Uses Chromium from `~/.cache/ms-playwright` (set `PLAYWRIGHT_BROWSERS_PATH` to override; run `npx playwright install chromium` if versions mismatch).
