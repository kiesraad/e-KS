import { test, expect } from '@playwright/test';
import { CandidateListsOverviewPage } from './pages/candidateListsOverviewPage';
import { SelectElectoralDistrictsPage } from './pages/selectElectoralDistrictsPage';
import { ManageCandidateListPage } from './pages/manageCandidateListPage';

test('add candidate list', async ({ page }) => {
  var candidateListsOverviewPage = new CandidateListsOverviewPage(page);
  await candidateListsOverviewPage.open();
  await candidateListsOverviewPage.addList();
  
  var selectElectoralDistrictsPage = new SelectElectoralDistrictsPage(page);
  await selectElectoralDistrictsPage.selectDistricts(['Drenthe', 'Groningen', 'Overijssel']); 
 
  var manageCandidateListPage = new ManageCandidateListPage(page);
  await manageCandidateListPage.addExistingCandidates(['Abdul Rahman, N.A. (Nadia)', 'Ali, F.A. (Fatima)', 'Alvarez, M.A. (Marco)'])

});
