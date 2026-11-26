import { test, expect } from '@playwright/test';

const KEY = 'rescue-demo-v1';
const cards = (page) => page.locator('.animal-card');

test.beforeEach(async ({ page }) => {
  await page.goto('/');
  await page.evaluate((k) => localStorage.removeItem(k), KEY);
  await page.reload();
});

test('roster shows three sample cards', async ({ page }) => {
  await expect(cards(page)).toHaveCount(3);
  for (const n of ['Biscuit', 'Pepper', 'Moose']) {
    await expect(page.locator('.animal-card', { hasText: n })).toBeVisible();
  }
});

test('filter chips narrow the roster', async ({ page }) => {
  await page.getByRole('button', { name: 'All', exact: true }).click();
  const all = await cards(page).count();
  expect(all).toBe(3);
  await page.getByRole('button', { name: 'Outcomes' }).click();
  expect(await cards(page).count()).toBeLessThan(all);
  await page.getByRole('button', { name: 'Available' }).click();
  await expect(cards(page).first().locator('.badge')).toHaveText('Available');
  await page.getByRole('button', { name: 'In care' }).click();
  await expect(cards(page).first()).toBeVisible();
});

test('Reset demo restores samples after storage is modified', async ({ page }) => {
  // Persist a valid blob first (fresh samples aren't written until a save/reset).
  page.once('dialog', (d) => d.accept());
  await page.getByRole('button', { name: 'Reset demo' }).click();
  await page.evaluate((k) => {
    const d = JSON.parse(localStorage.getItem(k)!);
    d.animals = [];
    localStorage.setItem(k, JSON.stringify(d));
  }, KEY);
  await page.reload();
  await page.getByRole('button', { name: 'All', exact: true }).click();
  await expect(cards(page)).toHaveCount(0);
  page.once('dialog', (d) => d.accept());
  await page.getByRole('button', { name: 'Reset demo' }).click();
  await expect(cards(page)).toHaveCount(3);
});

test('corrupt storage shows notice and falls back to samples', async ({ page }) => {
  await page.evaluate((k) => localStorage.setItem(k, '{bad'), KEY);
  await page.reload();
  await expect(page.getByText("Saved demo data couldn't be read and was reset")).toBeVisible();
  await page.getByRole('button', { name: 'All', exact: true }).click();
  await expect(cards(page)).toHaveCount(3);
});

test('banner and footer text are present', async ({ page }) => {
  await expect(page.getByText('Demo — entries stay in this browser only.')).toBeVisible();
  await expect(page.getByText('Demo only — not legal advice. Confirm current requirements with the Nebraska Department of Agriculture.')).toBeVisible();
});

test('no horizontal scroll at 375px', async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 800 });
  await page.reload();
  await expect(cards(page).first()).toBeVisible();
  const overflow = await page.evaluate(() => document.documentElement.scrollWidth > window.innerWidth);
  expect(overflow).toBe(false);
});
