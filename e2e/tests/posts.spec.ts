import { test, expect } from '@playwright/test';
import fs from 'fs';

const KEY = 'rescue-demo-v1';
const OUT = '../.superpowers/sdd/2026-11-25-rescue-intake-demo';

test.beforeEach(async ({ page }) => {
  await page.goto('/');
  await page.evaluate((k) => localStorage.removeItem(k), KEY);
  await page.goto('/#/animal/sample-biscuit/posts');
  await expect(page.getByRole('heading', { name: 'Upbeat' })).toBeVisible();
});

test('Biscuit shows three tone cards with prefilled posts', async ({ page }) => {
  for (const t of ['Upbeat', 'Heartfelt', 'Short']) {
    await expect(page.getByRole('heading', { name: t, exact: true })).toBeVisible();
  }
  const areas = page.locator('textarea.post-text');
  await expect(areas).toHaveCount(3);
  for (let i = 0; i < 3; i++) await expect(areas.nth(i)).toHaveValue(/Biscuit/);
  await expect(page.locator('canvas.share-canvas[data-ready="1"]')).toBeVisible();
  await page.setViewportSize({ width: 1280, height: 1400 });
  await page.screenshot({ path: `${OUT}/task-9-posts-1280.png`, fullPage: true });
});

test('Copy puts the (edited) text on the clipboard and confirms', async ({ page, context }) => {
  await context.grantPermissions(['clipboard-read', 'clipboard-write']);
  const area = page.locator('textarea.post-text').nth(1);
  await area.fill((await area.inputValue()) + ' Edited!');
  await page.getByRole('button', { name: 'Copy' }).nth(1).click();
  await expect(page.getByText('Copied!')).toBeVisible();
  const clip = await page.evaluate(() => navigator.clipboard.readText());
  expect(clip).toBe(await area.inputValue());
  expect(clip).toContain('Edited!');
  await expect(page.getByText('Copied!')).toBeHidden({ timeout: 4000 });
});

test('Download gives a 1080x1080 PNG named after the pet', async ({ page }) => {
  await expect(page.locator('canvas.share-canvas[data-ready="1"]')).toBeVisible();
  const [dl] = await Promise.all([
    page.waitForEvent('download'),
    page.getByRole('button', { name: 'Download image' }).click(),
  ]);
  expect(dl.suggestedFilename()).toBe('biscuit-adopt-me.png');
  const file = `${OUT}/task-9-share-biscuit.png`;
  await dl.saveAs(file);
  const buf = fs.readFileSync(file);
  expect(buf.subarray(1, 4).toString('latin1')).toBe('PNG');
  expect(buf.readUInt32BE(16)).toBe(1080);
  expect(buf.readUInt32BE(20)).toBe(1080);
});

test('sparse animal (name + species only) renders posts and image without "None"', async ({ page }) => {
  await page.goto('/#/new');
  await page.getByLabel('Name', { exact: true }).first().fill('Waffles');
  await page.getByRole('button', { name: 'Dog', exact: true }).click();
  await page.getByRole('button', { name: 'Save intake' }).click();
  await expect(page).toHaveURL(/#\/animal\/[^/]+$/);
  await page.getByRole('link', { name: 'Social posts' }).click();
  await expect(page.getByRole('heading', { name: 'Upbeat' })).toBeVisible();
  await expect(page.locator('canvas.share-canvas[data-ready="1"]')).toBeVisible();
  for (const v of await page.locator('textarea.post-text').evaluateAll((els) => els.map((e) => (e as HTMLTextAreaElement).value))) {
    expect(v).toContain('Waffles');
    expect(v).not.toContain('None');
  }
  await expect(page.locator('[role=alert]')).toHaveCount(0);
});

test('unknown id shows not-found', async ({ page }) => {
  await page.goto('/#/animal/nope/posts');
  await expect(page.getByText("We couldn't find that animal")).toBeVisible();
});

test('375px: no horizontal scroll', async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 800 });
  await expect(page.locator('canvas.share-canvas[data-ready="1"]')).toBeVisible();
  const overflow = await page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
  expect(overflow).toBeLessThanOrEqual(0);
  await page.screenshot({ path: `${OUT}/task-9-posts-375.png`, fullPage: true });
});
