import path from "node:path";
import { expect } from "@playwright/test";
import { test } from "./fixtures.ts";
import { CandidateListsOverviewPage } from "./pages/candidateListsOverviewPage.ts";
import { CsvImportExportPage } from "./pages/csvImportExportPage.ts";
import { ManageCandidateListPage } from "./pages/manageCandidateListPage.ts";

test.describe("import and export candidates with csv file", () => {
  async function navigateToCsvPage(page: Page) {
    await page.goto("/candidate-lists");
    await new CandidateListsOverviewPage(page).linkCandidateList
      .first()
      .click();
    await new ManageCandidateListPage(page).buttonCSV.click();
    const csvImportExport = new CsvImportExportPage(page);
    await expect(csvImportExport.headerImport).toBeVisible();
    return csvImportExport;
  }

  async function uploadCsvFile(
    page: Page,
    csvImportExport: CsvImportExportPage,
    filename: string,
  ) {
    await csvImportExport.buttonUpload.click();
    const [fileChooser] = await Promise.all([
      page.waitForEvent("filechooser"),
      csvImportExport.buttonContinue.click(),
    ]);
    await fileChooser.setFiles(path.join(__dirname, "testdata", filename));
  }

  test("import successful", async ({ login: page }) => {
    const csvImportExport = await navigateToCsvPage(page);
    await uploadCsvFile(
      page,
      csvImportExport,
      "candidate-list-export-nh-1.csv",
    );

    const manageCandidateListPage = new ManageCandidateListPage(page);
    await expect(manageCandidateListPage.headingCandidateList).toBeVisible();
    await expect(
      await manageCandidateListPage.getCandidateLocator("Groot, de"),
    ).toBeVisible();
  });

  test("import with errors", async ({ login: page }) => {
    const csvImportExport = await navigateToCsvPage(page);
    await uploadCsvFile(page, csvImportExport, "candidate-list-export-nh.csv");

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
    const csvImportExport = await navigateToCsvPage(page);
    const downloadPromise = page.waitForEvent("download");
    await csvImportExport.buttonDownload.click();
    const download = await downloadPromise;
    expect(download.suggestedFilename()).toMatch(
      /candidate-list-export-nh\.csv/,
    );
  });
});
