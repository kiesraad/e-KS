import { expect, Page } from "@playwright/test";
import { test } from "./fixtures.ts";
import { CsbOverviewPage } from "./pages/csbOverviewPage.ts";
import { CsbExaminationPage } from "./pages/csbExaminationPage.ts";
import { CsbImportPage } from "./pages/csbImportPage.ts";

  test("import political group", async ({ loginCSB: page }) => {
    const overviewPage = new CsbOverviewPage(page);
    const examinationPage = new CsbExaminationPage(page);
    const importPage = new CsbImportPage(page);

    await expect(overviewPage.headerElection).toBeVisible();
    await overviewPage.linkExamination.click();
    
    await expect(examinationPage.headerExamination).toBeVisible();
    await examinationPage.linkAddPoliticalGroup.click(); 
    
    await expect(importPage.headerImport).toBeVisible();
    await importPage.textfieldHashcode.fill("B948 38C0");
    await importPage.buttonImport.click();

    //await page.waitForURL("/csb/examination/**");

  });