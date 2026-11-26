import { test, expect } from '@playwright/test';
import fs from 'fs';

const KEY = 'rescue-demo-v1';
const OUT = '../.superpowers/sdd/2026-11-25-rescue-intake-demo';

test.beforeEach(async ({ page }) => {
  await page.goto('/');
  await page.evaluate((k) => localStorage.removeItem(k), KEY);
  await page.goto('/#/animal/sample-biscuit/sheet');
  await expect(page.locator('.ribbon')).toHaveText('Biscuit');
});

test('Biscuit sheet shows Overview labels and name ribbon, no Fun Facts', async ({ page }) => {
  const sheet = page.locator('.sheet');
  await expect(sheet.getByText('Overview')).toBeVisible();
  await expect(sheet.locator('dt', { hasText: 'Breed' })).toBeVisible();
  await expect(sheet.locator('dt', { hasText: 'Age' })).toBeVisible();
  await expect(sheet.getByText('Background')).toBeVisible();
  await expect(page.getByText('Fun Facts')).toHaveCount(0);
  await page.setViewportSize({ width: 1280, height: 1000 });
  await page.screenshot({ path: `${OUT}/task-8-sheet-1280.png`, fullPage: true });
});

test('print media hides banner, header and Print button', async ({ page }) => {
  await expect(page.getByRole('button', { name: 'Print' })).toBeVisible();
  await page.emulateMedia({ media: 'print' });
  await expect(page.locator('.banner')).toBeHidden();
  await expect(page.locator('.topbar')).toBeHidden();
  await expect(page.getByRole('button', { name: 'Print' })).toBeHidden();
  await expect(page.locator('.sheet')).toBeVisible();
});

test('prints to exactly one Letter page', async ({ page }) => {
  const pdf = await page.pdf({ format: 'Letter', printBackground: true });
  fs.writeFileSync(`${OUT}/task-8-sheet-print.pdf`, pdf);
  const pages = (pdf.toString('latin1').match(/\/Type\s*\/Page\b(?!s)/g) || []).length;
  expect(pages).toBe(1);
});

test('sparse animal has no None/Unknown/empty rows and shows the fallback', async ({ page }) => {
  await page.goto('/#/new');
  await page.getByLabel('Name', { exact: true }).first().fill('Waffles');
  await page.getByRole('button', { name: 'Dog', exact: true }).click();
  await page.getByRole('button', { name: 'Save intake' }).click();
  await expect(page).toHaveURL(/#\/animal\/[^/]+$/);
  await page.getByRole('link', { name: 'Fact sheet' }).click();
  await expect(page.locator('.ribbon')).toHaveText('Waffles');
  const sheet = page.locator('.sheet');
  await expect(sheet).toContainText('Ask us about Waffles!');
  const text = await sheet.innerText();
  expect(text).not.toMatch(/None|Unknown/);
  expect(await sheet.locator('dt').count()).toBe(await sheet.locator('dd').count());
  for (const dd of await sheet.locator('dd').all()) expect((await dd.innerText()).trim()).not.toBe('');
});

test('375px: no horizontal scroll', async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 800 });
  const over = await page.evaluate(() => document.documentElement.scrollWidth > window.innerWidth);
  expect(over).toBe(false);
  await page.screenshot({ path: `${OUT}/task-8-sheet-375.png`, fullPage: true });
});

test('unknown id shows not-found', async ({ page }) => {
  await page.goto('/#/animal/nope/sheet');
  await expect(page.getByRole('heading', { name: "We couldn't find that animal" })).toBeVisible();
});
