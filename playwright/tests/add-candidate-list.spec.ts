import { expect, test } from "@playwright/test";
import type { Candidate } from "./models/candidate";
import { CandidateListsOverviewPage } from "./pages/candidateListsOverviewPage";
import { ManageCandidateListPage } from "./pages/manageCandidateListPage";
import { SelectElectoralDistrictsPage } from "./pages/selectElectoralDistrictsPage";
import { randomName } from "./utils/random";

test.describe("add candidate list", async () => {
  test("add candidate list", async ({ page }) => {
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

  test("delete candidate list", async ({ page }) => {
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

  test("edit candidate list", async ({ page }) => {
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
});
