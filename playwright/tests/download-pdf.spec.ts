import { stat } from "node:fs/promises";
import { expect, type Page } from "@playwright/test";
import { test } from "./fixtures.ts";
import { CandidateListsOverviewPage } from "./pages/candidateListsOverviewPage.ts";
import { ManageCandidateListPage } from "./pages/manageCandidateListPage.ts";
import { EditListDetailsPage } from "./pages/editListDetailsPage.ts";
import { SubmitPage } from "./pages/submitPage.ts";

test.describe("download PDF", async () => {
  const existingCandidates = ["Akwasi", "Braber"];

  async function setupCandidateList(page: Page, district: string) {
    await page.goto("/candidate-lists");
    await new CandidateListsOverviewPage(page).buttonAddList.click();
    await new EditListDetailsPage(page).addDistricts([district]);
    await new ManageCandidateListPage(page).addExistingCandidates(
      existingCandidates,
    );
  }

  test("H1 NL", async ({ deleteExistingCandidateLists: page }) => {
    await setupCandidateList(page, "Gelderland");
    await page.goto("/submit");

    const downloadPromise = page.waitForEvent("download");
    await new SubmitPage(page).linkH1NLDownload.click();
    const download = await downloadPromise;

    expect(download.suggestedFilename()).toMatch(/model-h1-ge\.pdf/);
    expect((await stat(await download.path())).size).toBeGreaterThan(1024);
  });

  test("H1 FR", async ({ deleteExistingCandidateLists: page }) => {
    await setupCandidateList(page, "Gelderland");
    await page.goto("/submit");

    const downloadPromise = page.waitForEvent("download");
    await new SubmitPage(page).linkH1FRDownload.click();
    const download = await downloadPromise;

    expect(download.suggestedFilename()).toMatch(/model-h1-ge\.pdf/);
    expect((await stat(await download.path())).size).toBeGreaterThan(1024);
  });

  test("H3-1 NL", async ({ deleteExistingCandidateLists: page }) => {
    await setupCandidateList(page, "Groningen");
    await page.goto("/submit");

    const downloadPromise = page.waitForEvent("download");
    await new SubmitPage(page).linkH31NLDownload.click();
    const download = await downloadPromise;

    expect(download.suggestedFilename()).toMatch(/model-h3-1-gr\.pdf/);
    expect((await stat(await download.path())).size).toBeGreaterThan(1024);
  });

  test("H3-1 FR", async ({ deleteExistingCandidateLists: page }) => {
    await setupCandidateList(page, "Groningen");
    await page.goto("/submit");

    const downloadPromise = page.waitForEvent("download");
    await new SubmitPage(page).linkH31FRDownload.click();
    const download = await downloadPromise;

    expect(download.suggestedFilename()).toMatch(/model-h3-1-gr\.pdf/);
    expect((await stat(await download.path())).size).toBeGreaterThan(1024);
  });

  test("H4 NL", async ({ deleteExistingCandidateLists: page }) => {
    await setupCandidateList(page, "Utrecht");
    await page.goto("/submit");

    const downloadPromise = page.waitForEvent("download");
    await new SubmitPage(page).linkH4NLDownload.click();
    const download = await downloadPromise;

    expect(download.suggestedFilename()).toMatch(/model-h4-\(Utrecht\)\.pdf/);
    expect((await stat(await download.path())).size).toBeGreaterThan(1024);
  });

  test("H4 FR", async ({ deleteExistingCandidateLists: page }) => {
    await setupCandidateList(page, "Utrecht");
    await page.goto("/submit");

    const downloadPromise = page.waitForEvent("download");
    await new SubmitPage(page).linkH4FRDownload.click();
    const download = await downloadPromise;

    expect(download.suggestedFilename()).toMatch(/model-h4-\(Utert\)\.pdf/);
    expect((await stat(await download.path())).size).toBeGreaterThan(1024);
  });

  test("H9 NL", async ({ deleteExistingCandidateLists: page }) => {
    await setupCandidateList(page, "Zeeland");
    await page.goto("/submit");

    const downloadPromise = page.waitForEvent("download");
    await new SubmitPage(page).linkH9NLDownload.click();
    const download = await downloadPromise;

    expect(download.suggestedFilename()).toMatch(/model-h9-ze\.zip/);
    expect((await stat(await download.path())).size).toBeGreaterThan(1024);
  });

  test("H9 FR", async ({ deleteExistingCandidateLists: page }) => {
    await setupCandidateList(page, "Zeeland");
    await page.goto("/submit");

    const downloadPromise = page.waitForEvent("download");
    await new SubmitPage(page).linkH9FRDownload.click();
    const download = await downloadPromise;

    expect(download.suggestedFilename()).toMatch(/model-h9-ze\.zip/);
    expect((await stat(await download.path())).size).toBeGreaterThan(1024);
  });
});
