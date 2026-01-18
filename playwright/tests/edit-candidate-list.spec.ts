import { test, expect } from '@playwright/test';

test('edit candidate list', async ({ page }) => {
  await page.goto('/');

  await page.getByRole('link', { name: 'Candidate lists' }).click();
  await expect(page.getByRole('heading', { name: 'Candidate lists' })).toBeVisible();

  await page.getByRole('link', { name: 'Manage list' }).first().click();
  await expect(page.getByRole('heading', { name: 'Candidate list' })).toBeVisible();

  await page.getByRole('link', { name: 'List details' }).click();
  await expect(page.getByRole('heading', { name: 'Edit candidate list' })).toBeVisible();

  // edit list: deselect some electoral districts
  await page.getByRole('checkbox', { name: 'Drenthe' }).uncheck();
  await page.getByRole('checkbox', { name: 'Friesland' }).uncheck();
  await page.getByRole('checkbox', { name: 'Groningen' }).uncheck();
  await page.getByRole('button', { name: 'Save' }).click();
  await expect(page.getByRole('heading', { name: 'Candidate list' })).toBeVisible();

  await page.getByRole('link', { name: 'Candidate lists' }).click();
  await expect(page.getByRole('heading', { name: 'Candidate lists' })).toBeVisible();

  // Deselected electoral districts are no longer shown
  await expect(page.getByText('Electoral districts: Flevoland, Gelderland, Limburg, Noord-Brabant, Noord-')).toBeVisible();
});
