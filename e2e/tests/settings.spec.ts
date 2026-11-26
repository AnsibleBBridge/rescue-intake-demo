import { test, expect } from '@playwright/test';

import { KEY, OUT, pickSpecies } from './helpers';

test.beforeEach(async ({ page }) => {
  await page.goto('/');
  await page.evaluate((k) => localStorage.removeItem(k), KEY);
  await page.goto('/#/settings');
  await expect(page.getByRole('heading', { name: 'Settings' })).toBeVisible();
});

test('settings flow into the fact sheet, posts and share image', async ({ page }) => {
  await page.getByLabel('Rescue name').fill('Prairie Paws');
  await page.getByLabel('Phone').fill('402-555-0199');
  await page.getByLabel('Hashtags').fill('adopt nebraska');
  await page.getByRole('button', { name: 'Save' }).click();
  await expect(page.getByRole('status').filter({ hasText: 'Saved' })).toBeVisible();
  await page.setViewportSize({ width: 375, height: 812 });
  await page.screenshot({ path: `${OUT}/task-10-settings-375.png`, fullPage: true });
  await page.setViewportSize({ width: 1280, height: 720 });

  await page.goto('/#/animal/sample-clover/sheet');
  await expect(page.locator('.sheet-foot')).toContainText('402-555-0199');
  await expect(page.locator('.sheet-title')).toContainText('Prairie Paws');

  await page.goto('/#/animal/sample-clover/posts');
  await expect(page.locator('canvas.share-canvas[data-ready="1"]')).toBeAttached();
  const posts = await page.locator('textarea.post-text').evaluateAll((els) => els.map((e) => (e as HTMLTextAreaElement).value));
  expect(posts.some((v) => v.includes('Prairie Paws'))).toBe(true);
  expect(posts.some((v) => v.includes('#adopt #nebraska'))).toBe(true);

  // Changing settings again re-renders the image: data-ready drops, then returns.
  await page.getByRole('link', { name: 'Settings' }).click();
  await page.getByLabel('Rescue name').fill('Other Rescue');
  await page.getByRole('button', { name: 'Save' }).click();
  await page.goto('/#/animal/sample-clover/posts');
  await expect(page.locator('canvas.share-canvas[data-ready="1"]')).toBeAttached();
});

test('saved values persist across reload and Saved clears on edit', async ({ page }) => {
  await page.getByLabel('Website').fill('example.org');
  await page.getByRole('button', { name: 'Save' }).click();
  await expect(page.getByText('Saved', { exact: true })).toBeVisible();
  await page.getByLabel('Website').fill('example.net');
  await expect(page.getByText('Saved', { exact: true })).toBeHidden();
  await page.getByRole('button', { name: 'Save' }).click();
  await page.reload();
  await expect(page.getByLabel('Website')).toHaveValue('example.net');
});

test('Download image enables once the share image is ready', async ({ page }) => {
  await page.goto('/#/animal/sample-clover/posts');
  await expect(page.locator('canvas.share-canvas[data-ready="1"]')).toBeAttached();
  await expect(page.getByRole('button', { name: 'Download image' })).toBeEnabled();
});

test('unknown route shows Page not found with a roster link', async ({ page }) => {
  await page.goto('/#/bogus');
  await expect(page.getByRole('heading', { name: 'Page not found' })).toBeVisible();
  await page.getByRole('link', { name: 'Back to the roster' }).click();
  await expect(page).toHaveURL(/#\/$/);
});

test('fact sheet with no overview facts says details are coming soon', async ({ page }) => {
  await page.goto('/#/new');
  await page.getByLabel('Name', { exact: true }).first().fill('Waffles');
  await pickSpecies(page, 'Dog');
  await page.getByRole('button', { name: 'Save intake' }).click();
  await expect(page).toHaveURL(/#\/animal\/[^/]+$/);
  await page.getByRole('link', { name: /fact sheet/i }).click();
  await expect(page.getByText('Details coming soon.')).toBeVisible();
});
