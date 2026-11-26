import { test, expect } from '@playwright/test';
import { KEY, OUT, DOG_ID, seedOverdueDog } from './helpers';

// Dog-based tests run against a seeded 3-day-old dog: only dogs/cats have 48h deadlines.
const openDog = async (page) => {
  await seedOverdueDog(page);
  await page.goto(`/#/animal/${DOG_ID}`);
  await expect(page.getByRole('heading', { name: 'Rex', level: 1 })).toBeVisible();
};

const openSample = async (page, id: string, name: string) => {
  await page.goto('/');
  await page.evaluate((k) => localStorage.removeItem(k), KEY);
  await page.goto(`/#/animal/${id}`);
  await page.reload();
  await expect(page.getByRole('heading', { name, level: 1 })).toBeVisible();
};

test('a 3-day-old dog shows red overdue items, and rabies is only a reminder', async ({ page }) => {
  await openDog(page);
  expect(await page.locator('.ci.overdue').count()).toBeGreaterThanOrEqual(1);
  await expect(page.locator('.ci[data-key="rabies"]')).toContainText('Add when vaccinated');
  await expect(page.locator('.ci[data-key="rabies"]')).not.toHaveClass(/overdue/);
  await expect(page.locator('.ci.overdue').first()).toContainText('overdue by');
  // Dogs keep the 23 NAC 18 wording and don't get the farm note.
  await expect(page.getByRole('heading', { name: 'Records & reminders' })).toBeVisible();
  await expect(page.getByText('Nebraska has no single farm-sanctuary records law')).toHaveCount(0);
});

test('editing the dog to add a driver license turns the source item green', async ({ page }) => {
  await openDog(page);
  await expect(page.locator('.ci[data-key="source"]')).toHaveClass(/overdue/);
  await page.getByRole('button', { name: 'Edit', exact: true }).click();
  await page.getByLabel("Driver's license #").fill('N1234567');
  await page.getByRole('button', { name: 'Save changes' }).click();
  await expect(page.locator('.ci[data-key="source"]')).toHaveClass(/done/);
  await expect(page.getByRole('button', { name: 'Edit', exact: true })).toBeVisible();
});

test('status pills and outcome save are disabled while editing', async ({ page }) => {
  await openDog(page);
  await page.getByRole('button', { name: 'Outcome', exact: true }).click();
  await page.getByRole('button', { name: 'Edit', exact: true }).click();
  await page.getByLabel("Driver's license #").fill('N9');
  await expect(page.getByRole('button', { name: 'Available', exact: true })).toBeDisabled();
  await expect(page.getByRole('button', { name: 'Save Outcome' })).toBeDisabled();
  await expect(page.getByText('Save or cancel your edits first.').first()).toBeVisible();
  await page.getByRole('button', { name: 'Cancel' }).click();
  await expect(page.getByRole('button', { name: 'Available', exact: true })).toBeEnabled();
});

test('Clover profile shows all entered details', async ({ page }) => {
  await openSample(page, 'sample-clover', 'Clover');
  await expect(page.locator('dl.details dt', { hasText: /^Sex$/ })).toBeVisible();
  await expect(page.locator('dl.details dt', { hasText: /^Good with$/ })).toBeVisible();
  await expect(page.locator('dl.details dt', { hasText: /^Scrapie tag$/ })).toBeVisible();
  await expect(page.locator('dl.details dt', { hasText: /^Horns$/ })).toBeVisible();
  await expect(page.locator('dl.details')).not.toContainText(/None|Unknown/);
  await expect(page.locator('.profile-head .sub')).toContainText('Goat');
});

test('Cancel leaves the record unchanged', async ({ page }) => {
  await openDog(page);
  await page.getByRole('button', { name: 'Edit', exact: true }).click();
  await page.getByLabel("Driver's license #").fill('N1');
  await page.getByRole('button', { name: 'Cancel' }).click();
  await expect(page.locator('.ci[data-key="source"]')).toHaveClass(/overdue/);
});

test('outcome form adapts to the method and saves to the roster badge', async ({ page }) => {
  await openDog(page);
  await page.getByRole('button', { name: 'Outcome', exact: true }).click();
  const method = page.getByLabel('How did the animal leave?');
  await method.selectOption('Euthanized');
  await expect(page.getByLabel('Performed by')).toBeVisible();
  await expect(page.getByLabel('Method', { exact: true })).toBeVisible();
  await expect(page.getByLabel('Bill of sale / adoption contract #')).toHaveCount(0);
  await method.selectOption('Adopted');
  await expect(page.getByLabel('Recipient name')).toBeVisible();
  await expect(page.getByLabel('Recipient mailing address')).toBeVisible();
  await expect(page.getByLabel('Bill of sale / adoption contract #')).toBeVisible();
  await expect(page.getByLabel('Performed by')).toHaveCount(0);
  await page.getByLabel('Recipient name').fill('Sam Lee');
  await page.getByLabel('Recipient mailing address').fill('5 Elm St, Lincoln, NE');
  await page.getByLabel('Bill of sale / adoption contract #').fill('AC-1');
  await page.getByRole('button', { name: 'Save Outcome' }).click();
  await expect(page.locator('.ci[data-key="disposition"]')).toHaveClass(/done/);
  const stored = async () => page.evaluate((k) => JSON.parse(localStorage.getItem(k)!).animals[0].disposition, KEY);
  expect((await stored()).transfer_doc).toBe('AC-1');
  // Switching to Returned to owner clears the contract reference on save.
  await method.selectOption('Returned to owner');
  await expect(page.getByLabel('Bill of sale / adoption contract #')).toHaveCount(0);
  await page.getByRole('button', { name: 'Save Outcome' }).click();
  await expect.poll(async () => (await stored()).transfer_doc).toBeNull();
  await page.goto('/#/');
  await expect(page.locator('.animal-card', { hasText: 'Rex' }).locator('.badge')).toHaveText('Returned to owner');
});

test('Clover has no overdue items', async ({ page }) => {
  await openSample(page, 'sample-clover', 'Clover');
  await expect(page.locator('.ci')).not.toHaveCount(0);
  await expect(page.locator('.ci.overdue')).toHaveCount(0);
  await expect(page.locator('.ci:not(.done)')).toHaveCount(0);
});

test('unknown id shows a friendly not-found with a way back', async ({ page }) => {
  await openSample(page, 'sample-clover', 'Clover');
  await page.goto('/#/animal/bogus');
  await expect(page.getByText("We couldn't find that animal")).toBeVisible();
  await page.getByRole('link', { name: 'Back to the roster' }).click();
  await expect(page.locator('.animal-card')).toHaveCount(3);
});

test('375px: no horizontal scroll on the profile', async ({ page }) => {
  await openDog(page);
  await page.setViewportSize({ width: 375, height: 800 });
  await page.reload();
  await expect(page.getByRole('heading', { name: 'Rex', level: 1 })).toBeVisible();
  const over = await page.evaluate(() => document.documentElement.scrollWidth > window.innerWidth);
  expect(over).toBe(false);
  await page.screenshot({ path: `${OUT}/profile-dog-375.png`, fullPage: true });
});
