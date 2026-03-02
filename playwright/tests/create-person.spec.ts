import { expect, test } from "@playwright/test";
import type { AuthorisedPerson } from "./models/authorisedPerson";
import type { Candidate } from "./models/candidate";
import { AuthorisedPersonPage } from "./pages/authorisedPersonPage";
import { CandidateListsOverviewPage } from "./pages/candidateListsOverviewPage";
import { CorrespondenceAddressPage } from "./pages/correspondenceAddressPage";
import { CreatePersonPage } from "./pages/createPersonPage";
import { PersonsPage } from "./pages/personsPage";
import { randomName } from "./utils/random";

test.describe("create new person", async () => {
  const candidateAllFields: Candidate = {
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

  const candidateMinimalFields: Candidate = {
    initials: "D",
    lastName: `Duif ${randomName()}`,
  };
  const candidates = [
    { candidate: candidateAllFields, description: "with all fields" },
    {
      candidate: candidateMinimalFields,
      description: "with minimal required fields",
    },
  ];
  for (const { candidate, description } of candidates) {
    test(description, async ({ page }) => {
      await page.goto("/candidate-lists");
      await new CandidateListsOverviewPage(page).headingAllCandidates.click();
      const personsPage = new PersonsPage(page);
      await new PersonsPage(page).linkAddPerson.click();
      await new CreatePersonPage(page).setPersonalDetails(candidate);
      await new CorrespondenceAddressPage(page).setCorrespondenceAddress(
        candidate,
      );
      await expect(
        await personsPage.getCellLastName(candidate.lastName),
      ).toBeVisible();
    });
  }

  test("living outside NL requires authorised person", async ({ page }) => {
    await page.goto("/candidate-lists");
    await new CandidateListsOverviewPage(page).headingAllCandidates.click();

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

    const personsPage = new PersonsPage(page);
    await personsPage.linkAddPerson.click();
    await new CreatePersonPage(page).setPersonalDetails(candidate);
    await new AuthorisedPersonPage(page).setAuthorisedPerson(
      // biome-ignore lint/style/noNonNullAssertion: the test should fail if the authorised person is not set
      candidate.authorisedPerson!,
    );
    await expect(
      await personsPage.getCellLastName(candidate.lastName),
    ).toBeVisible();
  });
});
