import { test, expect } from '@playwright/test';

const KEY = 'rescue-demo-v1';

test.beforeEach(async ({ page }) => {
  await page.goto('/');
  await page.evaluate((k) => localStorage.removeItem(k), KEY);
  await page.goto('/#/animal/sample-pepper');
  await expect(page.getByRole('heading', { name: 'Pepper', level: 1 })).toBeVisible();
});

test('Pepper shows a red overdue item, and rabies is only a reminder', async ({ page }) => {
  expect(await page.locator('.ci.overdue').count()).toBeGreaterThanOrEqual(1);
  await expect(page.locator('.ci[data-key="rabies"]')).toContainText('Add when vaccinated');
  await expect(page.locator('.ci[data-key="rabies"]')).not.toHaveClass(/overdue/);
  await expect(page.locator('.ci.overdue').first()).toContainText('overdue by');
});

test('editing Pepper to add a driver license turns the source item green', async ({ page }) => {
  await expect(page.locator('.ci[data-key="source"]')).toHaveClass(/overdue/);
  await page.getByRole('button', { name: 'Edit', exact: true }).click();
  await page.getByLabel("Driver's license #").fill('N1234567');
  await page.getByRole('button', { name: 'Save changes' }).click();
  await expect(page.locator('.ci[data-key="source"]')).toHaveClass(/done/);
  await expect(page.getByRole('button', { name: 'Edit', exact: true })).toBeVisible();
});

test('status pills and outcome save are disabled while editing', async ({ page }) => {
  await page.getByRole('button', { name: 'Outcome', exact: true }).click();
  await page.getByRole('button', { name: 'Edit', exact: true }).click();
  await page.getByLabel("Driver's license #").fill('N9');
  await expect(page.getByRole('button', { name: 'Available', exact: true })).toBeDisabled();
  await expect(page.getByRole('button', { name: 'Save Outcome' })).toBeDisabled();
  await expect(page.getByText('Save or cancel your edits first.').first()).toBeVisible();
  await page.getByRole('button', { name: 'Cancel' }).click();
  await expect(page.getByRole('button', { name: 'Available', exact: true })).toBeEnabled();
});

test('Biscuit profile shows all entered details', async ({ page }) => {
  await page.goto('/#/animal/sample-biscuit');
  await expect(page.locator('dl.details dt', { hasText: /^Sex$/ })).toBeVisible();
  await expect(page.locator('dl.details dt', { hasText: /^Good with$/ })).toBeVisible();
  await expect(page.locator('dl.details')).not.toContainText(/None|Unknown/);
});

test('Cancel leaves the record unchanged', async ({ page }) => {
  await page.getByRole('button', { name: 'Edit', exact: true }).click();
  await page.getByLabel("Driver's license #").fill('N1');
  await page.getByRole('button', { name: 'Cancel' }).click();
  await expect(page.locator('.ci[data-key="source"]')).toHaveClass(/overdue/);
});

test('outcome form adapts to the method and saves to the roster badge', async ({ page }) => {
  await page.getByRole('button', { name: 'Outcome', exact: true }).click();
  const method = page.getByLabel('How did the animal leave?');
  await method.selectOption('Euthanized');
  await expect(page.getByLabel('Performed by')).toBeVisible();
  await expect(page.getByLabel('Method', { exact: true })).toBeVisible();
  await method.selectOption('Adopted');
  await expect(page.getByLabel('Recipient name')).toBeVisible();
  await expect(page.getByLabel('Recipient mailing address')).toBeVisible();
  await expect(page.getByLabel('Performed by')).toHaveCount(0);
  await page.getByLabel('Recipient name').fill('Sam Lee');
  await page.getByLabel('Recipient mailing address').fill('5 Elm St, Lincoln, NE');
  await page.getByRole('button', { name: 'Save Outcome' }).click();
  await expect(page.locator('.ci[data-key="disposition"]')).toHaveClass(/done/);
  await page.goto('/#/');
  await expect(page.locator('.animal-card', { hasText: 'Pepper' }).locator('.badge')).toHaveText('Adopted');
});

test('Biscuit has no overdue items', async ({ page }) => {
  await page.goto('/#/animal/sample-biscuit');
  await expect(page.getByRole('heading', { name: 'Biscuit', level: 1 })).toBeVisible();
  await expect(page.locator('.ci')).not.toHaveCount(0);
  await expect(page.locator('.ci.overdue')).toHaveCount(0);
});

test('unknown id shows a friendly not-found with a way back', async ({ page }) => {
  await page.goto('/#/animal/bogus');
  await expect(page.getByText("We couldn't find that animal")).toBeVisible();
  await page.getByRole('link', { name: 'Back to the roster' }).click();
  await expect(page.locator('.animal-card')).toHaveCount(3);
});

test('375px: no horizontal scroll on the profile', async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 800 });
  await page.reload();
  await expect(page.getByRole('heading', { name: 'Pepper', level: 1 })).toBeVisible();
  const over = await page.evaluate(() => document.documentElement.scrollWidth > window.innerWidth);
  expect(over).toBe(false);
  await page.screenshot({ path: '../.superpowers/sdd/2026-11-25-rescue-intake-demo/task-7-pepper-375.png', fullPage: true });
  await page.setViewportSize({ width: 1280, height: 900 });
  await page.screenshot({ path: '../.superpowers/sdd/2026-11-25-rescue-intake-demo/task-7-pepper-1280.png' });
});
