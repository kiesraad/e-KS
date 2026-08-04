import { expect } from "@playwright/test";
import { test } from "./fixtures.ts";
import { CandidateListsOverviewPage } from "./pages/candidateListsOverviewPage.ts";
import { CsvImportExportPage } from "./pages/csvImportExportPage.ts";
import { ManageCandidateListPage } from "./pages/manageCandidateListPage.ts";

test.describe("import and export candidates with csv file", () => {
  test.beforeEach("navigate to csv page", async ({ login: page }) => {
    await page.goto("/candidate-lists");
    await new CandidateListsOverviewPage(page).linkCandidateList
      .first()
      .click();
    await new ManageCandidateListPage(page).buttonCSV.click();
  });

  test("import successful", async ({ login: page }) => {
    const csvImportExport = new CsvImportExportPage(page);
    await csvImportExport.uploadCsvFile("candidate-list-export-nh-1.csv");
    const manageCandidateListPage = new ManageCandidateListPage(page);
    await expect(manageCandidateListPage.headingCandidateList).toBeVisible();
    await expect(
      await manageCandidateListPage.getCandidateLocator("Groot, de"),
    ).toBeVisible();
  });

  test("import with validation errors", async ({ login: page }) => {
    const csvImportExport = new CsvImportExportPage(page);
    await csvImportExport.uploadCsvFile("candidate-list-export-nh.csv");
    await expect(csvImportExport.textFailure).toBeVisible();

    const expectedErrors = [
      "'Geboortedatum'",
      "'Burgerservicenummer (BSN)'",
      "'Postcode'",
      "'Huisnummer'",
      "'Achternaam'",
      "'Voorvoegsel'",
      "'Geslacht'",
      "'Landcode'",
      "'Voorletters'",
    ];

    for (const error of expectedErrors) {
      await expect(
        await csvImportExport.getValidationErrors(error),
      ).toBeVisible();
    }
  });

  test("export", async ({ login: page }) => {
    const csvImportExport = new CsvImportExportPage(page);
    await csvImportExport.buttonDownload.evaluate((el) =>
      el.setAttribute("download", ""),
    );
    const [download] = await Promise.all([
      page.waitForEvent("download"),
      csvImportExport.buttonDownload.click(),
    ]);
    expect(download.suggestedFilename()).toMatch(/[0-9a-f]{8}-gr\.csv/);
  });
});
