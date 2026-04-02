import { expect } from "@playwright/test";
import { test } from "./fixtures.ts";
import type { Candidate } from "./models/candidate";
import type { ListSubmitter } from "./models/listSubmitter.ts";
import { CandidateListsOverviewPage } from "./pages/candidateListsOverviewPage";
import { ChooseListSubmitterPage } from "./pages/chooseListSubmitterPage.ts";
import { ManageCandidateListPage } from "./pages/manageCandidateListPage";
import { SelectElectoralDistrictsPage } from "./pages/selectElectoralDistrictsPage";
import { randomName } from "./utils/random";

test.describe("add candidate list", async () => {
  test("add candidate list", async ({ login: page }) => {
    await page.goto(`/dev/login?fixtures=true`);
    await page.goto("/candidate-lists");
    await new CandidateListsOverviewPage(page).buttonAddList.click();

    await new SelectElectoralDistrictsPage(page).selectDistricts([
      "Drenthe",
      "Groningen",
      "Overijssel",
    ]);

    const existingCandidates = ["Nagelhout", "Meerman", "Altena"];
    const manageCandidateListPage = new ManageCandidateListPage(page);
    await manageCandidateListPage.addExistingCandidates(existingCandidates);
    for (const existingCandidate of existingCandidates) {
      await expect(
        await manageCandidateListPage.getCandidateLocator(existingCandidate),
      ).toBeVisible();
    }

    const candidate: Candidate = {
      initials: "A",
      lastName: `Berg ${randomName()}`,
      firstName: "Anita",
      locality: "Utrecht",
    };
    const candidateTwo: Candidate = {
      initials: "B",
      lastName: `Beer ${randomName()}`,
      firstName: "Bert",
      locality: "Amsterdam",
    };

    await manageCandidateListPage.addNewCandidates([candidate, candidateTwo]);
    for (const newCandidate of [candidate, candidateTwo]) {
      await expect(
        await manageCandidateListPage.getCandidateLocator(
          newCandidate.lastName,
        ),
      ).toBeVisible();
    }
  });

  test("delete candidate list", async ({ login: page }) => {
    await page.goto("/candidate-lists");
    await new CandidateListsOverviewPage(page).buttonAddList.click();

    await new SelectElectoralDistrictsPage(page).selectDistricts([
      "Zeeland",
      "Limburg",
      "Overijssel",
    ]);
    const manageCandidateListPage = new ManageCandidateListPage(page);
    await manageCandidateListPage.removeList();
    for (const district of ["Zeeland", "Limburg", "Overijssel"]) {
      await expect(
        await manageCandidateListPage.getDistrictLocator(district),
      ).toHaveCount(0);
    }
  });

  test("edit electoral districts", async ({ login: page }) => {
    const candidateListsOverviewPage = new CandidateListsOverviewPage(page);
    await page.goto("/candidate-lists");
    await candidateListsOverviewPage.linkCandidateList.first().click();
    const manageCandidateListPage = new ManageCandidateListPage(page);
    await manageCandidateListPage.removeDistricts([
      "Utrecht",
      "Flevoland",
      "Kiescollege Saba",
    ]);

    await page.goto("/candidate-lists");

    for (const district of ["Utrecht", "Flevoland", "Kiescollege Saba"]) {
      await expect(
        await manageCandidateListPage.getDistrictLocator(district),
      ).toHaveCount(0);
    }
  });

  test("edit list submitters", async ({ login: page }) => {
    await page.goto("/candidate-lists");
    await new CandidateListsOverviewPage(page).linkCandidateList
      .first()
      .click();
    const manageCandidateListPage = new ManageCandidateListPage(page);
    await manageCandidateListPage.buttonEditList.click();
    await manageCandidateListPage.buttonNext.click();
    await page.waitForURL("**/list-submitter**");
    const chooseListSubmitterPage = new ChooseListSubmitterPage(page);

    const submitterOne: ListSubmitter = {
      initials: "G.H.",
      lastName: "Smit",
      lastNamePrefix: "van",
    };

    const submitterTwo: ListSubmitter = {
      initials: "I.J.",
      lastName: "Jong",
    };

    await chooseListSubmitterPage.removeSubstituteSubmitter(submitterOne);
    await chooseListSubmitterPage.selectSubstituteSubmitter(submitterTwo);
    await chooseListSubmitterPage.buttonSave.click();
    await manageCandidateListPage.buttonEditList.click();
    await manageCandidateListPage.buttonNext.click();

    await expect(
      chooseListSubmitterPage.getSubstituteSubmitterLocator(submitterOne),
    ).not.toBeChecked();
    await expect(
      chooseListSubmitterPage.getSubstituteSubmitterLocator(submitterTwo),
    ).toBeChecked();
  });
});
