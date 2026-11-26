import { test, expect, Page } from '@playwright/test';
import fs from 'fs';
import os from 'os';
import path from 'path';

const KEY = 'rescue-demo-v1';

test.beforeEach(async ({ page }) => {
  await page.goto('/');
  await page.evaluate((k) => localStorage.removeItem(k), KEY);
  await page.goto('/#/new');
  await expect(page.getByRole('heading', { name: 'New intake' })).toBeVisible();
});

const fillMinimum = async (page: Page, name: string) => {
  await page.getByLabel('Name', { exact: true }).first().fill(name);
  await page.getByRole('button', { name: 'Dog', exact: true }).click();
};

// A big, smooth (gradient + shapes) JPEG: realistic photo size without incompressible noise.
async function bigJpeg(page: Page, w: number, h: number): Promise<Buffer> {
  const b64 = await page.evaluate(([w, h]) => {
    const c = document.createElement('canvas');
    c.width = w; c.height = h;
    const g = c.getContext('2d')!;
    const grad = g.createLinearGradient(0, 0, w, h);
    grad.addColorStop(0, '#f6a'); grad.addColorStop(1, '#4af');
    g.fillStyle = grad; g.fillRect(0, 0, w, h);
    g.fillStyle = '#fc3'; g.beginPath(); g.arc(w / 2, h / 2, h / 4, 0, 7); g.fill();
    return c.toDataURL('image/jpeg', 0.95).split(',')[1];
  }, [w, h]);
  return Buffer.from(b64, 'base64');
}

test('only name and species saves, lands on profile route, roster has 4 cards', async ({ page }) => {
  await fillMinimum(page, 'Waffles');
  await page.getByRole('button', { name: 'Save intake' }).click();
  await expect(page).toHaveURL(/#\/animal\/[^/]+$/);
  await page.goto('/#/');
  await expect(page.locator('.animal-card')).toHaveCount(4);
  await expect(page.locator('.animal-card', { hasText: 'Waffles' })).toBeVisible();
});

test('empty name shows an error naming it and stays on the form', async ({ page }) => {
  await page.getByRole('button', { name: 'Dog', exact: true }).click();
  await page.getByRole('button', { name: 'Save intake' }).click();
  await expect(page.getByRole('alert')).toContainText('Name');
  await expect(page).toHaveURL(/#\/new$/);
});

test('large photo is downscaled below 250 KB in storage', async ({ page }) => {
  const buf = await bigJpeg(page, 3600, 2400);
  expect(buf.length).toBeGreaterThan(100_000);
  await fillMinimum(page, 'Photogenic');
  await page.locator('#photo-input').setInputFiles({ name: 'big.jpg', mimeType: 'image/jpeg', buffer: buf });
  await expect(page.getByAltText('Photo preview')).toBeVisible();
  await page.getByRole('button', { name: 'Save intake' }).click();
  await expect(page).toHaveURL(/#\/animal\//);
  const len = await page.evaluate((k) => {
    const d = JSON.parse(localStorage.getItem(k)!);
    return d.animals.find((a: any) => a.name === 'Photogenic').photo.length;
  }, KEY);
  expect(len).toBeGreaterThan(1000);
  expect(len).toBeLessThan(250_000);
});

test('non-image file shows the photo error and keeps no preview', async ({ page }) => {
  const f = path.join(os.tmpdir(), 'not-a-photo.txt');
  fs.writeFileSync(f, 'definitely not an image');
  await page.locator('#photo-input').setInputFiles(f);
  await expect(page.getByText("Couldn't read that photo")).toBeVisible();
  await expect(page.getByAltText('Photo preview')).toHaveCount(0);
});

test('blank optional fields are stored as null, and agency source saves', async ({ page }) => {
  await fillMinimum(page, 'Nullish');
  await page.getByRole('button', { name: 'Government agency' }).click();
  await page.getByLabel('Agency name').fill('Lincoln Animal Control');
  await page.getByRole('button', { name: 'Save intake' }).click();
  await expect(page).toHaveURL(/#\/animal\//);
  const a = await page.evaluate((k) => JSON.parse(localStorage.getItem(k)!).animals.find((x: any) => x.name === 'Nullish'), KEY);
  expect(a.story.breed).toBeNull();
  expect(a.identification.tag).toBeNull();
  expect(a.source.Agency.agency).toBe('Lincoln Animal Control');
});

test('form has no horizontal scroll at 375px', async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 800 });
  await page.reload();
  await expect(page.getByRole('heading', { name: 'New intake' })).toBeVisible();
  // Open the optional reveals too, so the widest layouts are measured.
  await page.getByLabel('Out of state?').check();
  await page.getByRole('button', { name: 'Add health entry' }).click();
  await page.getByLabel('Microchip #').fill('985112345678901');
  const overflow = await page.evaluate(() => document.documentElement.scrollWidth > window.innerWidth);
  expect(overflow).toBe(false);
});
