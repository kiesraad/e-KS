import { expect } from "@playwright/test";
import { test } from "./fixtures.ts";
import type { AuthorisedAgent } from "./models/authorisedAgent.ts";
import { AuthorisedAgentsPage } from "./pages/authorisedAgentsPage.ts";
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

    await submitPage.linkAuthorisedAgent.click();
    const authorisedAgentsPage = new AuthorisedAgentsPage(page);
    const agent: AuthorisedAgent = {
      initials: "K",
      lastName: "Jansen",
    };
    await authorisedAgentsPage.addAuthorisedAgent(agent);
    await page.goto("/submit");
    await expect(submitPage.linkAuthorisedAgent).not.toBeVisible();
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
