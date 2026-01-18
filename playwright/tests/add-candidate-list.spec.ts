import { test, expect } from '@playwright/test';

test('add candidate list', async ({ page }) => {
  await page.goto('/');

  await page.getByRole('link', { name: 'Candidate lists' }).click();
  await expect(page.getByRole('heading', { name: 'Candidate lists' })).toBeVisible();

  await page.getByRole('link', { name: 'Add list' }).click();
  await expect(page.getByText('Create candidate list')).toBeVisible();

  await page.getByRole('checkbox', { name: 'Drenthe' }).check();
  await page.getByRole('checkbox', { name: 'Groningen' }).check();
  await page.getByRole('checkbox', { name: 'Overijssel' }).check();
  await page.getByRole('button', { name: 'Save' }).click();
  await expect(page.getByRole('heading', { name: 'Candidate list' })).toBeVisible();

  // add exsisting person
  await page.getByRole('link', { name: 'Existing' }).click();
  await page.getByRole('row', { name: 'Abdul Rahman, N.A. (Nadia)' }).getByRole('button').click();
  await expect(page.getByRole('heading', { name: 'Candidate list' })).toBeVisible();
  await expect(page.getByRole('cell', { name: 'Abdul Rahman, N.A. (Nadia)' })).toBeVisible();

  // add more persons
  await page.getByRole('link', { name: 'Existing' }).click();
  await page.getByRole('row', { name: 'Ali, F.A. (Fatima) Eindhoven' }).getByRole('button').click();
  await expect(page.getByRole('cell', { name: 'Ali, F.A. (Fatima)' })).toBeVisible();

  await page.getByRole('link', { name: 'Existing' }).click();
  await page.getByRole('row', { name: 'Alvarez, M.A. (Marco)' }).getByRole('button').click();
  await expect(page.getByRole('cell', { name: 'Alvarez, M.A. (Marco)' })).toBeVisible();
});
