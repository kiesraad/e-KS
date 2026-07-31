import { expect, Page } from "@playwright/test";
import { test } from "./fixtures.ts";
import { CsbOverviewPage } from "./pages/csbOverviewPage.ts";
import { ExaminationPage } from "./pages/examinationPage.ts";
import { CsbImportPage } from "./pages/csbImportPage.ts";

  test("import political group", async ({ login: page }) => {
    const csbOverviewPage = new CsbOverviewPage(page);
    const examinationPage = new ExaminationPage(page);
    const csbImportPage = new CsbImportPage(page);

    await expect(csbOverviewPage.headerElection).toBeVisible();
    await csbOverviewPage.linkExamination.click();
    
    await expect(examinationPage.headerExamination).toBeVisible();
    await examinationPage.linkAddPoliticalGroup.click(); 
    
    await expect(csbImportPage.headerImport).toBeVisible();
    await csbImportPage.textfieldHashcode.fill("B948 38C0");
    await csbImportPage.buttonImport.click();

    await page.waitForURL("/csb/examination/**");

  });