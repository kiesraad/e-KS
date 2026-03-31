import { test as base, type Page } from "@playwright/test";
import { AuthorisedAgentsPage } from "./pages/authorisedAgentsPage";
import { CandidateListsOverviewPage } from "./pages/candidateListsOverviewPage";
import { ListSubmittersPage } from "./pages/listSubmittersPage";
import { ManageCandidateListPage } from "./pages/manageCandidateListPage";
import { SubstituteSubmittersPage } from "./pages/substituteSubmittersPage";

type Fixtures = {
  deleteExistingGeneralInformation: Page;
  deleteExistingCandidateLists: Page;
};

export const test = base.extend<Fixtures>({
  deleteExistingGeneralInformation: async ({ page }, use) => {
    await page.goto("/political-group/authorised-agents");
    await new AuthorisedAgentsPage(page).deleteExistingAuthorisedAgents();

    await page.goto("/political-group/list-submitters");
    await new ListSubmittersPage(page).deleteExistingListSubmitters();

    await new SubstituteSubmittersPage(
      page,
    ).deleteExistingSubstituteSubmitters();

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
