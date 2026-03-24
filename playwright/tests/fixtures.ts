import { test as base, type Page } from "@playwright/test";
import { AuthorisedAgentsPage } from "./pages/authorisedAgentsPage";
import { CandidateListsOverviewPage } from "./pages/candidateListsOverviewPage";
import { ListSubmittersPage } from "./pages/listSubmittersPage";
import { ManageCandidateListPage } from "./pages/manageCandidateListPage";
import { SubstituteSubmittersPage } from "./pages/substituteSubmittersPage";
import { randomName } from "./utils/random";

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
    await page.goto(`/dev/login?name=${randomName()}&fixtures=true`);
    await page.goto("/candidate-lists");
    const candidateListsOverviewPage = new CandidateListsOverviewPage(page);

    while (true) {
      const candidateLists =
        await candidateListsOverviewPage.linkCandidateList.all();
      const visibleList = await Promise.all(
        candidateLists.map(async (item) => ({
          item,
          visible: await item.isVisible(),
        })),
      );

      const firstVisible = visibleList.find((entry) => entry.visible);
      if (!firstVisible) break;

      await firstVisible.item.click();
      await new ManageCandidateListPage(page).removeList();
      await page.goto("/candidate-lists");
    }

    await use(page);
  },
});
