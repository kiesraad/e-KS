import { expect, test } from "@playwright/test";
import { PoliticalGroupPage } from "./pages/politicalGroupPage";
import type { PoliticalGroup } from "./models/politicalGroup";
import type { AuthorisedAgent } from "./models/authorisedAgent";
import type { ListSubmitter} from "./models/listSubmitter"
import { AuthorisedAgentsPage } from "./pages/authorisedAgentsPage";
import { ListSumbittersPage } from "./pages/listSubmittersPage";


test.describe("provide general information for political group", async () => {

  test.beforeEach("start application and login", async ({ page }) => {
    //this will later be added when login is enabled
})

  test("provide general information for political group", async ({ page }) => {
    const politicalGroupPage = new PoliticalGroupPage(page);
    await politicalGroupPage.open();
    await politicalGroupPage.selectHasMoreThan16Seats("Ja");
    await politicalGroupPage.setRegisteredDesignation("TP");
    await politicalGroupPage.setStatutoryName("De Testpartij");

  })

  test("provide authorised agent", async ({ page }) => {
  
    const authorisedAgentsPage = new AuthorisedAgentsPage(page);
    await authorisedAgentsPage.open();
    await authorisedAgentsPage.deleteExistingAuthorisedAgents();
    const agentOne: AuthorisedAgent = {
        initials: "K",
        lastNamePrefix: "de",
        lastName: "Koek",
      };
      const agentTwo: AuthorisedAgent = {
        initials: "E",
        lastName: "Ekster",
      };
      await authorisedAgentsPage.addAuthorisedAgent([agentOne, agentTwo]);

      for (const agent of [agentOne, agentTwo]) {
        var agentLastName = agent.lastNamePrefix ? `${agent.lastNamePrefix} ${agent.lastName}` : agent.lastName
        await expect(authorisedAgentsPage.getAgentLocator(agentLastName)).toBeVisible();
      }
  })

  test("provide list submitter", async ({ page }) => {

    const listSumbittersPage = new ListSumbittersPage(page);
    await listSumbittersPage.open();
    await listSumbittersPage.deleteExistingListSubmitters();
    const submitterOne: ListSubmitter = {
        initials: "B",
        lastNamePrefix: "de",
        lastName: "Beer",
      };
      const submitterTwo: ListSubmitter = {
        initials: "O",
        lastName: "Olifant",
      };
      await listSumbittersPage.addListSubmitter([submitterOne, submitterTwo]);

      for (const submitter of [submitterOne, submitterTwo]) {
        var submitterLastName = submitter.lastNamePrefix ? `${submitter.lastNamePrefix} ${submitter.lastName}` : submitter.lastName
        await expect(listSumbittersPage.getSubmitterLocator(submitterLastName)).toBeVisible();
      }
})

}) 

