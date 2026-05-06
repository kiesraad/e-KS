import { stat } from "node:fs/promises";
import { expect, type Page } from "@playwright/test";
import { test } from "./fixtures.ts";
import { CandidateListsOverviewPage } from "./pages/candidateListsOverviewPage.ts";
import { EditListDetailsPage } from "./pages/editListDetailsPage.ts";
import { ManageCandidateListPage } from "./pages/manageCandidateListPage.ts";
import { SubmitPage } from "./pages/submitPage.ts";

test.describe("download documents", async () => {
  const existingCandidates = ["Akwasi", "Braber"];

  async function setupCandidateList(page: Page, district: string) {
    await page.goto("/candidate-lists");
    await new CandidateListsOverviewPage(page).buttonAddList.click();
    await new EditListDetailsPage(page).addDistricts([district]);
    await new ManageCandidateListPage(page).addExistingCandidates(
      existingCandidates,
    );
  }

  test("download links", async ({ deleteExistingCandidateLists: page }) => {
    await setupCandidateList(page, "Gelderland");
    await page.goto("/submit");

    const downloadPromise = page.waitForEvent("download");
    await new SubmitPage(page).linkDownloadNl.click();
    const download = await downloadPromise;

    expect(download.suggestedFilename()).toMatch("documents.zip");
    expect((await stat(await download.path())).size).toBeGreaterThan(1024);
  });

  test("EML 210", async ({ deleteExistingCandidateLists: page }) => {
    await setupCandidateList(page, "Friesland");
    await page.goto("/submit");

    const downloadPromise = page.waitForEvent("download");
    const submitPage = new SubmitPage(page);
    await submitPage.linkEML210Download.evaluate((el) =>
      el.setAttribute("download", ""),
    );
    await submitPage.linkEML210Download.click();
    const download = await downloadPromise;

    expect(download.suggestedFilename()).toMatch(/eml210\.eml\.xml/);
    expect((await stat(await download.path())).size).toBeGreaterThan(1024);
  });
});
