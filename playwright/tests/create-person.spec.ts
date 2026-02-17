import { test } from "@playwright/test";
import type { AuthorisedPerson } from "./models/authorisedPerson";
import type { Candidate } from "./models/candidate";
import { CandidateListsOverviewPage } from "./pages/candidateListsOverviewPage";
import { PersonsPage } from "./pages/personsPage";
import { randomName } from "./utils/random";

test("create new person", async ({ page }) => {
  const candidateListsOverviewPage = new CandidateListsOverviewPage(page);
  await candidateListsOverviewPage.open();
  await candidateListsOverviewPage.managePersons();

  const personsPage = new PersonsPage(page);
  const candidate: Candidate = {
    initials: "H",
    lastName: `Jansen ${randomName()}`,
    lastNamePrefix: "van",
    firstName: "Henk",
    gender: "male",
    dateOfBirth: "12-08-1977",
    postalCode: "6512EX",
    houseNumber: "26",
    streetName: "Castellastraat",
    locality: "Nijmegen",
  };
  const candidateTwo: Candidate = {
    initials: "D",
    lastName: `Duif ${randomName()}`,
  };
  await personsPage.addPersons([candidate, candidateTwo]);

  await personsPage.checkPerson([candidate, candidateTwo]);
});

test("create new person living outside NL requires authorised person", async ({
  page,
}) => {
  const candidateListsOverviewPage = new CandidateListsOverviewPage(page);
  await candidateListsOverviewPage.open();
  await candidateListsOverviewPage.managePersons();

  const personsPage = new PersonsPage(page);
  const authorisedPerson: AuthorisedPerson = {
    initials: "C",
    lastName: "Winter",
  };
  const candidate: Candidate = {
    initials: "H",
    lastName: `Jansen ${randomName()}`,
    countryCode: "VA",
    authorisedPerson: authorisedPerson,
  };

  await personsPage.addPersons([candidate]);

  await personsPage.checkPerson([candidate]);
});
