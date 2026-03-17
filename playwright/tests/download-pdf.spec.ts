import { expect, type Page } from "@playwright/test";
import { test } from "./fixtures.ts";
import { CandidateListsOverviewPage } from "./pages/candidateListsOverviewPage.ts";
import { ManageCandidateListPage } from "./pages/manageCandidateListPage.ts";
import { SelectElectoralDistrictsPage } from "./pages/selectElectoralDistrictsPage.ts";
import { SubmitPage } from "./pages/submitPage.ts";

test.describe("download PDF", async () => {
  const existingCandidates = ["Akwasi", "Braber"];

  async function setupAndDownload(
    page: Page,
    district: string,
    clickDownloadLink: (submitPage: SubmitPage) => Promise<void>,
  ) {
    await page.goto("/candidate-lists");
    await new CandidateListsOverviewPage(page).buttonAddList.click();
    await new SelectElectoralDistrictsPage(page).selectDistricts([district]);
    await new ManageCandidateListPage(page).addExistingCandidates(
      existingCandidates,
    );
    await page.goto("/submit");

    const downloadPromise = page.waitForEvent("download");
    await clickDownloadLink(new SubmitPage(page));
    return downloadPromise;
  }

  test("H1 NL", async ({ deleteExistingCandidateLists: page }) => {
    const download = await setupAndDownload(page, "Drenthe", (s) =>
      s.linkH1NLDownload.click(),
    );
    expect(download.suggestedFilename()).toMatch(/model-h1-dr\.pdf/);
  });

  test("H1 FR", async ({ deleteExistingCandidateLists: page }) => {
    const download = await setupAndDownload(page, "Drenthe", (s) =>
      s.linkH1FRDownload.click(),
    );
    expect(download.suggestedFilename()).toMatch(/model-h1-dr\.pdf/);
  });

  test("H3-1 NL", async ({ deleteExistingCandidateLists: page }) => {
    const download = await setupAndDownload(page, "Groningen", (s) =>
      s.linkH31NLDownload.click(),
    );
    expect(download.suggestedFilename()).toMatch(/model-h3-1-gr\.pdf/);
  });

  test("H3-1 FR", async ({ deleteExistingCandidateLists: page }) => {
    const download = await setupAndDownload(page, "Groningen", (s) =>
      s.linkH31FRDownload.click(),
    );
    expect(download.suggestedFilename()).toMatch(/model-h3-1-gr\.pdf/);
  });

  test("H4 NL", async ({ deleteExistingCandidateLists: page }) => {
    const download = await setupAndDownload(page, "Utrecht", (s) =>
      s.linkH4NLDownload.click(),
    );
    expect(download.suggestedFilename()).toMatch(/model-h4-\(Utrecht\)\.pdf/);
  });

  test("H4 FR", async ({ deleteExistingCandidateLists: page }) => {
    const download = await setupAndDownload(page, "Utrecht", (s) =>
      s.linkH4FRDownload.click(),
    );
    expect(download.suggestedFilename()).toMatch(/model-h4-\(Utert\)\.pdf/);
  });

  test("H9 NL", async ({ deleteExistingCandidateLists: page }) => {
    const download = await setupAndDownload(page, "Zeeland", (s) =>
      s.linkH9NLDownload.click(),
    );
    expect(download.suggestedFilename()).toMatch(/model-h9-ze\.zip/);
  });

  test("H9 FR", async ({ deleteExistingCandidateLists: page }) => {
    const download = await setupAndDownload(page, "Zeeland", (s) =>
      s.linkH9FRDownload.click(),
    );
    expect(download.suggestedFilename()).toMatch(/model-h9-ze\.zip/);
  });
});
