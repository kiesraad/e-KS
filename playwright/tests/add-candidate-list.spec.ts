// Use cases:
// politieke groepering maakt een lijst aan
// Politieke groepering vinkt aan voor welke gebieden de lijst geldig is
// Politieke groepering vult de lijst door personen toe te voegen
// Er ontbreken kandidaten, de lijstinleveraar vult de missende kandidaten in

import { expect } from "@playwright/test";
import { test } from "./fixtures.ts";
import type { Candidate } from "./models/candidate";
import { CandidateListsOverviewPage } from "./pages/pg/candidateListsOverviewPage";
import { EditListDetailsPage } from "./pages/pg/editListDetailsPage.ts";
import { ManageCandidateListPage } from "./pages/pg/manageCandidateListPage";
import { randomName } from "./utils/random";

test.describe("add candidate list", async () => {
  test("add candidate list", async ({ login: page }) => {
    await page.goto("/candidate-lists");
    await new CandidateListsOverviewPage(page).buttonAddList.click();

    await new EditListDetailsPage(page).addDistricts([
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

  test("add candidate list provincial council", async ({
    provincialCouncilElection: page,
  }) => {
    await page.goto("/candidate-lists");
    await new CandidateListsOverviewPage(page).buttonAddList.click();

    await new EditListDetailsPage(page).addDistricts(["Amsterdam", "Haarlem"]);

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
      locality: "Haarlem",
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

  test("add candidate list water authority", async ({
    waterAuthorityElection: page,
  }) => {
    await page.goto("/candidate-lists");
    await new CandidateListsOverviewPage(page).buttonAddList.click();
    const manageCandidateListPage = new ManageCandidateListPage(page);
    const candidate: Candidate = {
      initials: "A",
      lastName: `Berg ${randomName()}`,
      firstName: "Anita",
      locality: "Breukelen",
    };
    const candidateTwo: Candidate = {
      initials: "B",
      lastName: `Beer ${randomName()}`,
      firstName: "Bert",
      locality: "Loenen aan de Vecht",
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

    await new EditListDetailsPage(page).addDistricts([
      "Zeeland",
      "Limburg",
      "Overijssel",
    ]);
    await expect(
      page.getByRole("heading", { name: "Kandidatenlijst" }),
    ).toBeVisible();
    const candidatelistURL = page.url();
    const manageCandidateListPage = new ManageCandidateListPage(page);
    await manageCandidateListPage.removeList();
    await page.goto(candidatelistURL);
    await expect(
      page.getByRole("heading", { name: "Not found" }),
    ).toBeVisible();
  });

  test("delete candidate list provincial council", async ({
    provincialCouncilElection: page,
  }) => {
    await page.goto("/candidate-lists");
    await new CandidateListsOverviewPage(page).buttonAddList.click();

    await new EditListDetailsPage(page).addDistricts([
      "Den Helder",
      "Amsterdam",
    ]);
    await expect(
      page.getByRole("heading", { name: "Kandidatenlijst" }),
    ).toBeVisible();
    const candidatelistURL = page.url();
    const manageCandidateListPage = new ManageCandidateListPage(page);
    await manageCandidateListPage.removeList();
    await page.goto(candidatelistURL);
    await expect(
      page.getByRole("heading", { name: "Not found" }),
    ).toBeVisible();
  });

  test("edit electoral districts", async ({ login: page }) => {
    await page.goto("/candidate-lists");
    await new CandidateListsOverviewPage(page).buttonAddList.click();

    await new EditListDetailsPage(page).addDistricts([
      "Zeeland",
      "Groningen",
      "Overijssel",
    ]);
    await expect(
      page.locator("//li/a[normalize-space()='10. Zeeland']"),
    ).toBeVisible();
    await expect(
      page.locator("//li/a[normalize-space()='1. Groningen']"),
    ).toBeVisible();
    await expect(
      page.locator("//li/a[normalize-space()='4. Overijssel']"),
    ).toBeVisible();
    const manageCandidateListPage = new ManageCandidateListPage(page);
    await manageCandidateListPage.removeDistricts(["Zeeland", "Overijssel"]);
    await expect(
      page.locator("//li/a[normalize-space()='10. Zeeland']"),
    ).not.toBeVisible();
    await expect(
      page.locator("//li/a[normalize-space()='1. Groningen']"),
    ).toBeVisible();
    await expect(
      page.locator("//li/a[normalize-space()='4. Overijssel']"),
    ).not.toBeVisible();
  });

  test("edit electoral districts provincial council", async ({
    provincialCouncilElection: page,
  }) => {
    await page.goto("/candidate-lists");
    await new CandidateListsOverviewPage(page).buttonAddList.click();

    await new EditListDetailsPage(page).addDistricts(["Haarlem", "Den Helder"]);
    await expect(
      page.locator("//li/a[normalize-space()='10. Haarlem']"),
    ).toBeVisible();
    await expect(
      page.locator("//li/a[normalize-space()='11. Den Helder']"),
    ).toBeVisible();
    await expect(
      page.locator("//li/a[normalize-space()='9. Amsterdam']"),
    ).not.toBeVisible();
    const manageCandidateListPage = new ManageCandidateListPage(page);
    await manageCandidateListPage.removeDistricts(["Haarlem"]);
    await manageCandidateListPage.addDistricts(["Amsterdam"]);
    await expect(
      page.locator("//li/a[normalize-space()='10. Haarlem']"),
    ).not.toBeVisible();
    await expect(
      page.locator("//li/a[normalize-space()='11. Den Helder']"),
    ).toBeVisible();
    await expect(
      page.locator("//li/a[normalize-space()='9. Amsterdam']"),
    ).toBeVisible();
  });
});
