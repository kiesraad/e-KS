import { test as base, expect, type Page } from "@playwright/test";
import { CandidateListsOverviewPage } from "./pages/candidateListsOverviewPage";
import { ManageCandidateListPage } from "./pages/manageCandidateListPage";
import { OverviewPage } from "./pages/overviewPage";
import { SelectElectionPage } from "./pages/selectElectionPage";

type Fixtures = {
  login: Page;
  noExistingData: Page;
  deleteExistingCandidateLists: Page;
  provincialCouncilElection: Page;
  waterAuthorityElection: Page;
};

export const test = base.extend<Fixtures>({
  login: async ({ page }, use) => {
    await page.goto("/dev/login?fixtures=true");
    await use(page);
  },

  noExistingData: async ({ page }, use) => {
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
        await new CandidateListsOverviewPage(page).buttonAddList.waitFor();
      }
    }

    await use(page);
  },

  provincialCouncilElection: async ({ page }, use) => {
    await page.goto(`/dev/login?fixtures=true`);
    await new OverviewPage(page).linkLogout.click();
    const selectElectionPage = new SelectElectionPage(page);
    await expect(selectElectionPage.HeaderElections).toBeVisible();
    await selectElectionPage.dropdownElections.selectOption("PS27");
    await selectElectionPage.dropdownProvinces.selectOption("NH");
    await selectElectionPage.checkboxFixtures.check();
    await selectElectionPage.buttonContinue.click();

    await use(page);
  },

  waterAuthorityElection: async ({ page }, use) => {
    await page.goto(`/dev/login?fixtures=true`);
    await new OverviewPage(page).linkLogout.click();
    const selectElectionPage = new SelectElectionPage(page);
    await expect(selectElectionPage.HeaderElections).toBeVisible();
    await selectElectionPage.dropdownElections.selectOption(
      "Waterschapsverkiezingen 2027",
    );
    await selectElectionPage.dropdownWaterAuthorities.selectOption(
      "Amstel, Gooi en Vecht",
    );
    await selectElectionPage.checkboxFixtures.uncheck();
    await selectElectionPage.buttonContinue.click();

    await use(page);
  },
});
