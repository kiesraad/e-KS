import { test } from "@playwright/test";
import type { Candidate } from "./models/candidate";
import { CandidateListsOverviewPage } from "./pages/candidateListsOverviewPage";
import { PersonsPage } from "./pages/personsPage";

test("create new person", async ({ page }) => {
  const candidateListsOverviewPage = new CandidateListsOverviewPage(page);
  await candidateListsOverviewPage.open();
  await candidateListsOverviewPage.managePersons();

  const personsPage = new PersonsPage(page);
  const candidate: Candidate = {
    initials: "H",
    lastName: "Jansen",
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
    lastName: "Duif",
  };
  await personsPage.addPersons([candidate, candidateTwo]);

  await personsPage.checkPerson([candidate, candidateTwo]);
});
