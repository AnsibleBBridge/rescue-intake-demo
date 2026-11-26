import { Page } from '@playwright/test';

export const KEY = 'rescue-demo-v1';
export const OUT = '../.superpowers/sdd/2026-11-25-farm-animals-v2';
export const DOG_ID = 'seed-rex';

/** Pick a species in the intake form's select by its visible label. */
export const pickSpecies = (page: Page, label: string) => page.getByLabel('Species', { exact: true }).selectOption(label);

/**
 * Replace storage with one v1-shaped Dog received 3 days ago (no v2 fields at all), then reload.
 * The UI can't backdate an intake, and dogs are the only species with 48h deadlines, so this is
 * how the e2e suite keeps coverage of the red "overdue by" display. Source lacks a driver's
 * license, so the source and identification items are overdue.
 */
export async function seedOverdueDog(page: Page) {
  await page.goto('/');
  await page.evaluate(([k, id]) => {
    const pad = (n: number) => String(n).padStart(2, '0');
    const d = new Date(Date.now() - 3 * 24 * 3600 * 1000);
    const ts = `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}:00`;
    const blob = {
      version: 1,
      animals: [{
        id, name: 'Rex', species: 'Dog', sex: 'Male', received_at: ts,
        source: { Person: { name: 'Pat Doe', address: '1 Main St, Omaha, NE', drivers_license: '' } },
        out_of_state: false, health_certificate: null,
        identification: { tag: null, cage_card: null, tattoo: null, microchip: null },
        health: [],
        story: {
          breed: 'Lab mix', age_text: null, birth_date: null, color_markings: null, weight_lbs: null,
          altered: 'Unknown', good_with_dogs: 'Yes', good_with_cats: 'Unknown', good_with_kids: 'Yes',
          house_trained: 'Yes', energy: null, personality: null, special_needs: null, adoption_fee: null,
        },
        photo: null, status: 'Intake', status_changed_at: ts, disposition: null,
      }],
      settings: { rescue_name: 'Your Rescue Name', phone: '(402) 555-0100', email: 'adopt@example.org', website: '', hashtags: ['#AdoptDontShop'] },
    };
    localStorage.setItem(k as string, JSON.stringify(blob));
  }, [KEY, DOG_ID]);
  await page.reload();
}
