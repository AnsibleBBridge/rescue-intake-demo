import { test, expect } from '@playwright/test';

const KEY = 'rescue-demo-v1';
const OUT = '../.superpowers/sdd/2026-11-25-rescue-intake-demo';

// Every page must fit a 375px phone without sideways scrolling.
const pages: [string, string][] = [
  ['roster', '/#/'],
  ['intake', '/#/new'],
  ['profile', '/#/animal/sample-biscuit'],
  ['sheet', '/#/animal/sample-biscuit/sheet'],
  ['posts', '/#/animal/sample-biscuit/posts'],
  ['settings', '/#/settings'],
];

for (const [name, url] of pages) {
  test(`375px: ${name} has no horizontal scroll`, async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 812 });
    await page.goto('/');
    await page.evaluate((k) => localStorage.removeItem(k), KEY);
    await page.goto(url);
    await expect(page.locator('main.page > *').first()).toBeVisible();
    if (name === 'posts') await expect(page.locator('canvas.share-canvas[data-ready="1"]')).toBeVisible();
    const overflow = await page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
    expect(overflow).toBeLessThanOrEqual(0);
    await page.screenshot({ path: `${OUT}/task-11-${name}-375.png`, fullPage: true });
  });
}
