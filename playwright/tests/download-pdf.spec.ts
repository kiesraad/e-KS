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

  test("download export", async ({ deleteExistingCandidateLists: page }) => {
    await setupCandidateList(page, "Gelderland");
    await page.goto("/submit");

    const downloadPromise = page.waitForEvent("download");
    await new SubmitPage(page).linkDownloadNl.click();
    const download = await downloadPromise;

    expect(download.suggestedFilename()).toMatch(/^[a-z0-9-]+-v\d+\.zip$/);
    expect((await stat(await download.path())).size).toBeGreaterThan(1024);

    await expect(new SubmitPage(page).linkDownloadFry).not.toBeVisible();
  });

  test("download frisian export", async ({
    provincialCouncilFrisianElection: page,
  }) => {
    await page.goto("/submit");

    const downloadPromise = page.waitForEvent("download");
    await new SubmitPage(page).linkDownloadNl.click();
    const download = await downloadPromise;

    expect(download.suggestedFilename()).toMatch(/^[a-z0-9-]+-v\d+\.zip$/);
    expect((await stat(await download.path())).size).toBeGreaterThan(1024);

    const downloadPromise2 = page.waitForEvent("download");
    await new SubmitPage(page).linkDownloadFry.click();
    const download2 = await downloadPromise2;

    expect(download2.suggestedFilename()).toMatch(/^[a-z0-9-]+-v\d+-fry\.zip$/);
    expect((await stat(await download2.path())).size).toBeGreaterThan(1024);
  });
});
