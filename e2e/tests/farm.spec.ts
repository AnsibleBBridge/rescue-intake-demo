import { test, expect, Page } from '@playwright/test';
import fs from 'fs';
import { KEY, OUT, DOG_ID, pickSpecies, seedOverdueDog } from './helpers';

test.beforeEach(async ({ page }) => {
  await page.goto('/');
  await page.evaluate((k) => localStorage.removeItem(k), KEY);
  await page.reload();
});

const stored = (page: Page, name: string) =>
  page.evaluate(([k, n]) => JSON.parse(localStorage.getItem(k)!).animals.find((a: any) => a.name === n), [KEY, name]);

const newIntake = async (page: Page, name: string, species: string) => {
  await page.goto('/#/new');
  await expect(page.getByRole('heading', { name: 'New intake' })).toBeVisible();
  await page.getByLabel('Name', { exact: true }).first().fill(name);
  await pickSpecies(page, species);
};

test('goat intake shows scrapie + horns, not coggins; switching to duck hides them and saves no scrapie tag', async ({ page }) => {
  await newIntake(page, 'Gus', 'Goat');
  await expect(page.getByLabel('Scrapie tag #')).toBeVisible();
  await expect(page.getByRole('group', { name: 'Horns' })).toBeVisible();
  await expect(page.getByLabel('Coggins date')).toHaveCount(0);
  await page.getByLabel('Scrapie tag #').fill('NE-123');
  await page.getByRole('button', { name: 'Horned', exact: true }).click();
  // Farm intake: reminders note, not the 48-hour deadline note.
  await expect(page.getByText('We found no Nebraska records law written for farm-animal rescues').first()).toBeVisible();

  await pickSpecies(page, 'Duck');
  await expect(page.getByLabel('Scrapie tag #')).toHaveCount(0);
  await expect(page.getByRole('group', { name: 'Horns' })).toHaveCount(0);
  await expect(page.getByRole('group', { name: 'Lays eggs' })).toBeVisible();
  await page.getByRole('button', { name: 'Save intake' }).click();
  await expect(page).toHaveURL(/#\/animal\//);
  const a = await stored(page, 'Gus');
  expect(a.species).toBe('Duck');
  expect(a.scrapie_tag).toBeNull();
  expect(a.story.horns).toBeNull();
});

test('Jasper shows Coggins and ownership as info reminders, nothing red', async ({ page }) => {
  await page.goto('/#/animal/sample-jasper');
  await expect(page.getByRole('heading', { name: 'Jasper', level: 1 })).toBeVisible();
  for (const key of ['coggins', 'ownership_doc']) {
    const row = page.locator(`.ci[data-key="${key}"]`);
    await expect(row).toHaveClass(/reminder/);
    await expect(row).toContainText('ℹ️');
    await expect(row).not.toHaveClass(/overdue/);
  }
  await expect(page.locator('.ci.overdue')).toHaveCount(0);
  await expect(page.locator('.ci.reminder')).toHaveCount(2);
  await expect(page.locator('.profile-head .sub')).toContainText('Donkey');
});

test('Other + Alpaca shows "Alpaca" on the roster card and "alpaca" in posts', async ({ page }) => {
  await newIntake(page, 'Pablo', 'Other');
  await page.getByLabel('What kind of animal?').fill('Alpaca');
  await page.getByRole('button', { name: 'Save intake' }).click();
  await expect(page).toHaveURL(/#\/animal\//);
  await page.goto('/#/');
  const card = page.locator('.animal-card', { hasText: 'Pablo' });
  await expect(card.locator('.sub')).toContainText('Alpaca');
  await expect(card).not.toContainText('Other');
  await card.click();
  await page.getByRole('link', { name: 'Social posts' }).click();
  await expect(page.getByRole('heading', { name: 'Upbeat' })).toBeVisible();
  const posts = await page.locator('textarea.post-text').evaluateAll((els) => els.map((e) => (e as HTMLTextAreaElement).value));
  expect(posts.some((v) => /alpaca/i.test(v))).toBe(true);
  for (const v of posts) expect(v).not.toMatch(/\bOther\b|None|Unknown|  /);
});

test('a v1 stored blob (Dog, no v2 fields) loads with no reset notice', async ({ page }) => {
  await seedOverdueDog(page);
  await expect(page.locator('.animal-card', { hasText: 'Rex' })).toBeVisible();
  await expect(page.locator('.animal-card')).toHaveCount(1);
  await expect(page.getByText("couldn't be read")).toHaveCount(0);
  await expect(page.locator('.notice')).toHaveCount(0);
  await expect(page.locator('.animal-card .sub')).toContainText('Dog');
  // Opening and re-saving it works too.
  await page.goto(`/#/animal/${DOG_ID}`);
  await expect(page.locator('.ci.overdue').first()).toContainText('overdue by');
});

test('375px: intake with Equine fields open has no horizontal scroll', async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 800 });
  await newIntake(page, 'Dobbin', 'Horse');
  await expect(page.getByLabel('Height (hands)')).toBeVisible();
  await expect(page.getByLabel('Coggins date')).toBeVisible();
  await page.getByLabel('Out of state?').check();
  await page.getByRole('button', { name: 'Add health entry' }).click();
  const overflow = await page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
  expect(overflow).toBeLessThanOrEqual(0);
});

test('each family gets its own placeholder emoji on the roster and a silhouette on the sheet', async ({ page }) => {
  const cases: [string, string][] = [
    ['Horse', '\u{1F434}'], ['Goat', '\u{1F410}'], ['Chicken', '\u{1F414}'], ['Rabbit', '\u{1F430}'],
    ['Dog', '\u{1F436}'], ['Cat', '\u{1F431}'], ['Other', '\u{1F43E}'],
  ];
  const paths = new Set<string>();
  for (const [species, emoji] of cases) {
    const name = `T-${species}`;
    await newIntake(page, name, species);
    await page.getByRole('button', { name: 'Save intake' }).click();
    await expect(page).toHaveURL(/#\/animal\//);
    await expect(page.locator('.profile-head .ph')).toHaveText(emoji);
    await page.getByRole('link', { name: 'Fact sheet' }).click();
    paths.add((await page.locator('.sheet svg.sil path').getAttribute('d'))!);
  }
  expect(paths.size).toBe(cases.length);
});

test('v2 sample screenshots', async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 812 });
  await page.goto('/#/');
  await expect(page.locator('.animal-card')).toHaveCount(3);
  await page.screenshot({ path: `${OUT}/v2-roster-375.png`, fullPage: true });

  await newIntake(page, 'Gus', 'Goat');
  await page.getByRole('button', { name: 'Horned', exact: true }).click();
  await page.screenshot({ path: `${OUT}/v2-intake-goat-375.png`, fullPage: true });

  await page.goto('/#/animal/sample-jasper');
  await expect(page.getByRole('heading', { name: 'Jasper', level: 1 })).toBeVisible();
  await page.screenshot({ path: `${OUT}/v2-jasper-375.png`, fullPage: true });

  await page.setViewportSize({ width: 1280, height: 1000 });
  await page.goto('/#/animal/sample-clover/sheet');
  await expect(page.locator('.ribbon')).toHaveText('Clover');
  await page.screenshot({ path: `${OUT}/v2-sheet-clover-1280.png`, fullPage: true });

  await page.goto('/#/animal/sample-jasper/posts');
  await expect(page.locator('canvas.share-canvas[data-ready="1"]')).toBeAttached();
  await expect(page.locator('img.share-img')).toBeVisible();
  const [dl] = await Promise.all([page.waitForEvent('download'), page.getByRole('button', { name: 'Download image' }).click()]);
  await dl.saveAs(`${OUT}/v2-share-jasper.png`);
  expect(fs.readFileSync(`${OUT}/v2-share-jasper.png`).subarray(1, 4).toString('latin1')).toBe('PNG');
});

test('horse height is hands.inches: 14.4 is rejected with a visible message, 14.2 saves unchanged', async ({ page }) => {
  await newIntake(page, 'Dobbin', 'Horse');
  const h = page.getByLabel('Height (hands)');
  await expect(h).toHaveAttribute('step', '0.1');
  await h.fill('14.4');
  await page.getByRole('button', { name: 'Save intake' }).click();
  await expect(page.getByRole('alert').filter({ hasText: 'Height is hands.inches' })).toBeVisible();
  await expect(page).toHaveURL(/#\/new/);
  await h.fill('14.2');
  await page.getByRole('button', { name: 'Save intake' }).click();
  await expect(page).toHaveURL(/#\/animal\//);
  expect((await stored(page, 'Dobbin')).story.height_hands).toBeCloseTo(14.2, 5);
});

test('farm intake hides the "own ID" line; dog intake keeps it', async ({ page }) => {
  await newIntake(page, 'Gus', 'Goat');
  await expect(page.getByText('Each animal needs its own ID.')).toHaveCount(0);
  await pickSpecies(page, 'Dog');
  await expect(page.getByText('Each animal needs its own ID.')).toBeVisible();
});
