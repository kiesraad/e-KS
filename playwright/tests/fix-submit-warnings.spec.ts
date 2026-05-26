import { expect } from "@playwright/test";
import { test } from "./fixtures.ts";
import type { NameAuthorisation } from "./models/nameAuthorisation.ts";
import { NameAuthorisationPage } from "./pages/nameAuthorisationPage.ts";
import { CreatePersonPage } from "./pages/createPersonPage.ts";
import { ManageCandidateListPage } from "./pages/manageCandidateListPage.ts";
import { PoliticalGroupPage } from "./pages/politicalGroupPage.ts";
import { SubmitPage } from "./pages/submitPage.ts";

test.describe("fix submit warnings", async () => {
  test("general information", async ({ noExistingData: page }) => {
    const submitPage = new SubmitPage(page);
    const politicalGroupPage = new PoliticalGroupPage(page);

    await page.goto("/submit");
    await submitPage.linkRegisteredDesignation.click();
    await expect(politicalGroupPage.headerGeneralInformation).toBeVisible();
    await politicalGroupPage.setRegisteredDesignation("Test");
    await politicalGroupPage.save();
    await page.goto("/submit");
    await expect(submitPage.linkRegisteredDesignation).not.toBeVisible();

    await submitPage.linkNoLegalName.click();
    const nameAuthorisationPage = new NameAuthorisationPage(page);
    const authorisation: NameAuthorisation = {
      initials: "K",
      lastName: "Jansen",
      legalName: "Kiesraad Demo Partij",
    };
    await nameAuthorisationPage.addNameAuthorisation(authorisation);
    await page.goto("/submit");
    await expect(submitPage.linkNoLegalName).not.toBeVisible();
  });

  test("candidates", async ({ login: page }) => {
    const submitPage = new SubmitPage(page);
    const createPersonPage = new CreatePersonPage(page);

    await page.goto("/submit");
    await submitPage.linkBSN.click();
    await createPersonPage.checkboxNoBSN.check();
    await createPersonPage.buttonNext.click();
    await page.goto("/submit");
    await expect(submitPage.linkBSN).not.toBeVisible();

    await submitPage.linkIncorrectDate.first().click();
    await createPersonPage.textfieldYearOfBirth.fill("1925");
    await createPersonPage.buttonNext.click();
    await page.goto("/submit");

    await submitPage.linkIncorrectDate.click();
    await createPersonPage.textfieldYearOfBirth.fill("1990");
    await createPersonPage.buttonNext.click();
    await page.goto("/submit");
    await expect(submitPage.linkIncorrectDate).not.toBeVisible();
  });

  test("candidate lists", async ({ login: page }) => {
    const submitPage = new SubmitPage(page);
    const manageCandidateListPage = new ManageCandidateListPage(page);

    await page.goto("/submit");
    await submitPage.linkTooManyCandidates.first().click();
    await manageCandidateListPage.deleteCandidates([
      "Nagelhout",
      "Meerman",
      "Altena",
      "Smit",
      "Bruin",
    ]);
    await page.goto("/submit");
    await expect(submitPage.linkTooManyCandidates).not.toBeVisible();
  });
});
