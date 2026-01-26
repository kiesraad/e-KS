import { test, expect } from '@playwright/test';
import { CandidateListsOverviewPage } from './pages/candidateListsOverviewPage';
import { PersonsPage } from './pages/personsPage';
import { Candidate } from './models/candidate';

test('create new person', async ({ page }) => {
  var candidateListsOverviewPage = new CandidateListsOverviewPage(page);
  await candidateListsOverviewPage.open();
  await candidateListsOverviewPage.managePersons();

  var personsPage = new PersonsPage(page);
    var candidate: Candidate = {
      initials: 'H',
      lastName: 'Jansen',
      lastNamePrefix: 'van',
      firstName: 'Henk',
      gender: 'male',
      dateOfBirth: '12-08-1977',
      postalCode: '6512EX',
      houseNumber: '26',
      streetName: 'Castellastraat',
      locality: 'Nijmegen'
    }
      var candidateTwo: Candidate = {
      initials: 'D',
      lastName: 'Duif',
    }
    await personsPage.addPersons([candidate, candidateTwo]);

    
});
