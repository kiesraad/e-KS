import { test as base, type Page } from "@playwright/test";
import { AuthorisedAgentsPage } from "./pages/authorisedAgentsPage";
import { ListSubmittersPage } from "./pages/listSubmittersPage";
import { SubstituteSubmittersPage } from "./pages/substituteSubmittersPage";

type Fixtures = {
  deleteExistingData: Page;
};

export const test = base.extend<Fixtures>({
  deleteExistingData: async ({ page }, use) => {
    await page.goto("/political-group/authorised-agents");
    await new AuthorisedAgentsPage(page).deleteExistingAuthorisedAgents();

    await page.goto("/political-group/list-submitters");
    await new ListSubmittersPage(page).deleteExistingListSubmitters();

    await new SubstituteSubmittersPage(
      page,
    ).deleteExistingSubstituteSubmitters();

    await use(page);
  },
});
