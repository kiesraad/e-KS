import { test, expect } from '@playwright/test';

test('test', async ({ page }) => {
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

  // create new person
  await page.getByRole('link', { name: 'New' }).click();
  await page.getByRole('textbox', { name: 'Initials *' }).fill('H.A.H.A');
  await page.getByLabel('Last name' ).fill('Jansen');
//  await page.locator('input[name="last_name"]').fill('Jansen');
  await page.getByRole('textbox', { name: 'First name' }).fill('Henk');
  await page.getByLabel('Gender').selectOption('male');
  await page.getByRole('textbox', { name: 'Date of birth' }).fill('01-01-1970');
  await page.getByRole('button', { name: 'Save' }).click();

  // add address details
  await page.getByRole('textbox', { name: 'Postal code' }).fill('6512EX');
  await page.getByRole('textbox', { name: 'House number', exact: true }).fill('26');
  await page.getByRole('textbox', { name: 'House number', exact: true }).press('Tab');

  // address lookup
  await expect(page.getByRole('textbox', { name: 'Street name' })).toHaveValue('Castellastraat');
  await expect(page.getByRole('combobox', { name: 'Locality' })).toHaveValue('Nijmegen');

  // save address
  await page.getByRole('button', { name: 'Save' }).click();

  // verify person is added to candidate list
  await expect(page.getByRole('cell', { name: 'Jansen, H.A.H.A. (Henk)' })).toBeVisible();
  await expect(page.getByRole('cell', { name: 'Nijmegen' })).toBeVisible();

});
