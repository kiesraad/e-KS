import { expect, type Page } from "@playwright/test";
import { test } from "./fixtures.ts";
import { SubmitPage } from "./pages/submitPage.ts";
import { PoliticalGroupPage } from "./pages/politicalGroupPage.ts";
import { CreatePersonPage } from "./pages/createPersonPage.ts";

test.describe("fix submit warnings", async () => {
  test("general information", async ({ noExistingData : page }) => {
    await page.goto("/submit");
    const submitPage = new SubmitPage(page);
    await submitPage.linkRegisteredDesignation.click();
    const politicalGroupPage = new PoliticalGroupPage(page);
    await expect(politicalGroupPage.headerGeneralInformation).toBeVisible();
    await politicalGroupPage.setRegisteredDesignation("Test");
    await politicalGroupPage.save();
    await page.goto("/submit");
    await expect(submitPage.linkRegisteredDesignation).not.toBeVisible();    
  });

  test("candidates", async ({ login : page }) => {
    await page.goto("/submit");
    const submitPage = new SubmitPage(page);
    await submitPage.linkBSN.click();
    const createPersonPage = new CreatePersonPage(page);
    await expect(createPersonPage.textfieldInitials).toBeVisible();
    await createPersonPage.checkboxNoBSN.check();
    await createPersonPage.buttonNext.click();
    await page.goto("/submit");
    await expect(submitPage.linkBSN).not.toBeVisible();    
  });
});

