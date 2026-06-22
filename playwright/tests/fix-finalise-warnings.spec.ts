import { expect } from "@playwright/test";
import { test } from "./fixtures.ts";
import type { NameAuthorisation } from "./models/nameAuthorisation.ts";
import { CreatePersonPage } from "./pages/createPersonPage.ts";
import { FinalisePage } from "./pages/finalisePage.ts";
import { ListSubmittersPage } from "./pages/listSubmittersPage.ts";
import { ManageCandidateListPage } from "./pages/manageCandidateListPage.ts";
import { NameAuthorisationPage } from "./pages/nameAuthorisationPage.ts";
import { OverviewPage } from "./pages/overviewPage.ts";
import { PoliticalGroupPage } from "./pages/politicalGroupPage.ts";

test.describe("fix submit warnings", async () => {
  test("general information", async ({ noExistingData: page }) => {
    const finalisePage = new FinalisePage(page);
    const politicalGroupPage = new PoliticalGroupPage(page);
    const listSubmittersPage = new ListSubmittersPage(page);
    const nameAuthorisationPage = new NameAuthorisationPage(page);
    const overviewPage = new OverviewPage(page);
    const authorisation: NameAuthorisation = {
      initials: "K",
      lastName: "Jansen",
      legalName: "Kiesraad Demo Partij",
    };

    await page.goto("/finalise");
    await finalisePage.linkRegisteredDesignation.click();
    await expect(politicalGroupPage.headerGeneralInformation).toBeVisible();
    await politicalGroupPage.setRegisteredDesignation("Test");
    await politicalGroupPage.save();
    await nameAuthorisationPage.buttonNext.click();
    await listSubmittersPage.buttonNext.click();
    await overviewPage.linkFinalise.click();
    await page.waitForURL("/finalise");
    await expect(finalisePage.linkRegisteredDesignation).not.toBeVisible();

    await finalisePage.linkNoLegalName.click();
    await nameAuthorisationPage.editNameAuthorisation([authorisation]);
    await page.waitForURL("/finalise");
    await expect(finalisePage.linkNoLegalName).not.toBeVisible();
  });

  test("candidates", async ({ login: page }) => {
    const finalisePage = new FinalisePage(page);
    const createPersonPage = new CreatePersonPage(page);

    await page.goto("/finalise");
    await finalisePage.linkBSN.click();
    await createPersonPage.checkboxNoBSN.check();
    await createPersonPage.buttonNext.click();
    await page.waitForURL("/finalise");
    await expect(finalisePage.linkBSN).not.toBeVisible();

    await finalisePage.linkIncorrectDate.first().click();
    await createPersonPage.textfieldYearOfBirth.fill("1925");
    await createPersonPage.buttonNext.click();
    await page.waitForURL("/finalise");

    await finalisePage.linkIncorrectDate.click();
    await createPersonPage.textfieldYearOfBirth.fill("1990");
    await createPersonPage.buttonNext.click();
    await page.waitForURL("/finalise");
    await expect(finalisePage.linkIncorrectDate).not.toBeVisible();
  });

  test("candidate lists", async ({ login: page }) => {
    const finalisePage = new FinalisePage(page);
    const manageCandidateListPage = new ManageCandidateListPage(page);

    await page.goto("/finalise");
    await finalisePage.linkTooManyCandidates.first().click();
    await manageCandidateListPage.deleteCandidates([
      "Nagelhout",
      "Meerman",
      "Altena",
      "Smit",
      "Bruin",
    ]);
    await manageCandidateListPage.buttonFinalise.click();
    await page.waitForURL("/finalise");
    await expect(finalisePage.linkTooManyCandidates).not.toBeVisible();
  });
});
