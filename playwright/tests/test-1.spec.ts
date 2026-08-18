import { test, expect } from '@playwright/test';

test('test', async ({ page }) => {
  await page.goto('http://localhost:3000/login');
  await page.getByRole('button', { name: 'Inloggen' }).click();
  await page.getByText('Login als CSB').click();
  await page.getByRole('button', { name: 'Verder' }).click();
  await page.getByRole('link', { name: 'Fase 2 Sluit op 25-04-27' }).click();
  await page.getByRole('cell', { name: 'Kiesraad Demo' }).click();
  await page.getByLabel('Onderzoek afronden of').locator('span').click();
  await page.getByLabel('Onderzoek afronden of').locator('span').click();
  await page.getByLabel('Onderzoek afronden of').locator('span').click();
  await page.getByRole('link', { name: 'Terug' }).click();
  await page.getByText('Goedgekeurd').click();
});