import { test as base, type Page } from "@playwright/test";
import { CandidateListsOverviewPage } from "./pages/candidateListsOverviewPage";
import { ManageCandidateListPage } from "./pages/manageCandidateListPage";
import { SelectElectionPage } from "./pages/selectElectionPage";


type Fixtures = {
  login: Page;
  loginCSB: Page;
  noExistingData: Page;
  noExistingDataCSB: Page;  
  deleteExistingCandidateLists: Page;
  provincialCouncilElection: Page;
  provincialCouncilFrisianElection: Page;
  waterAuthorityElection: Page;
  emlCSB: Page; 
};

export const test = base.extend<Fixtures>({
  login: async ({ page }, use) => {
    await page.goto("/dev/login?fixtures=true");
    await use(page);
  },

  loginCSB: async ({ page }, use) => {
    await page.goto("/dev/login?fixtures=true&csb=true");
    await use(page);
  },

  noExistingData: async ({ page }, use) => {
    await page.goto("/dev/login?fixtures=false");
    await use(page);
  },

  noExistingDataCSB: async ({ page }, use) => {
    await page.goto("/dev/login?fixtures=false&csb=true");
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
    await page.goto("/dev/login?select_election=true");
    const selectElectionPage = new SelectElectionPage(page);
    await selectElectionPage.dropdownElections.selectOption("PS27");
    await selectElectionPage.dropdownProvinces.selectOption("NH");
    await selectElectionPage.checkboxFixtures.check();
    await Promise.all([
      page.waitForURL("/"),
      selectElectionPage.buttonContinue.click(),
    ]);

    await use(page);
  },

  provincialCouncilFrisianElection: async ({ page }, use) => {
    await page.goto("/dev/login?select_election=true");
    const selectElectionPage = new SelectElectionPage(page);
    await selectElectionPage.dropdownElections.selectOption("PS27");
    await selectElectionPage.dropdownProvinces.selectOption("FR");
    await selectElectionPage.checkboxFixtures.check();
    await Promise.all([
      page.waitForURL("/"),
      selectElectionPage.buttonContinue.click(),
    ]);

    await use(page);
  },

  waterAuthorityElection: async ({ page }, use) => {
    await page.goto("/dev/login?select_election=true");
    const selectElectionPage = new SelectElectionPage(page);
    await selectElectionPage.dropdownElections.selectOption(
      "Waterschapsverkiezingen 2027",
    );
    await selectElectionPage.dropdownWaterAuthorities.selectOption(
      "Amstel, Gooi en Vecht",
    );
    await selectElectionPage.checkboxFixtures.uncheck();
    await Promise.all([
      page.waitForURL("/"),
      selectElectionPage.buttonContinue.click(),
    ]);

    await use(page);
  },

});
