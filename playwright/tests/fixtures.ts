import { test as base, type Page } from "@playwright/test";
import { CandidateListsOverviewPage } from "./pages/candidateListsOverviewPage";
import { ManageCandidateListPage } from "./pages/manageCandidateListPage";

type Fixtures = {
  login: Page;
  noExistingGeneralInformation: Page;
  deleteExistingCandidateLists: Page;
};

export const test = base.extend<Fixtures>({
  login: async ({ page }, use) => {
    await page.goto("/dev/login?fixtures=true");
    await use(page);
  },

  noExistingGeneralInformation: async ({ page }, use) => {
    await page.goto("/dev/login?fixtures=false");
    await use(page);
  },

  deleteExistingCandidateLists: async ({ page }, use) => {
    await page.goto(`/dev/login?fixtures=true`);
    await page.goto("/candidate-lists");
    const candidateListsOverviewPage = new CandidateListsOverviewPage(page);

    const hrefs =
      await candidateListsOverviewPage.linkCandidateList.evaluateAll((links) =>
        links.map((link) => link.getAttribute("href")),
      );

    for (const href of hrefs) {
      if (href) {
        await page.goto(href);
        await new ManageCandidateListPage(page).removeList();
      }
    }

    await use(page);
  },
});
