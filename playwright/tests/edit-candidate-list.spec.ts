import { test, expect } from '@playwright/test';
import { CandidateListsOverviewPage } from './pages/candidateListsOverviewPage';
import { ManageCandidateListPage } from './pages/manageCandidateListPage';

test('edit candidate list', async ({ page }) => {
  var candidateListsOverviewPage = new CandidateListsOverviewPage(page);
  await candidateListsOverviewPage.open();
  await candidateListsOverviewPage.manageList();
  
  var manageCandidateListPage = new ManageCandidateListPage(page);
  await manageCandidateListPage.removeDistricts(['Drenthe', 'Friesland', 'Groningen']);

  await candidateListsOverviewPage.open();
  await candidateListsOverviewPage.checkDistricts(['Drenthe', 'Friesland', 'Groningen']);

  await candidateListsOverviewPage.manageList();
  await manageCandidateListPage.addDistricts(['Drenthe', 'Friesland', 'Groningen']);

  await candidateListsOverviewPage.open();
  await expect(page.getByText('Electoral districts: All')).toBeVisible();

});
