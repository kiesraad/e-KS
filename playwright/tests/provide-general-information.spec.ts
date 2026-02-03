import { test } from "@playwright/test";
import { PoliticalGroupPage } from "./pages/politicalGroupPage";
import type { PoliticalGroup } from "./models/politicalGroup";
import type { AuthorisedAgent } from "./models/authorisedAgent";
import { AuthorisedAgentsPage } from "./pages/authorisedAgentsPage";

test("provide general information for political group", async ({ page }) => {
  const politicalGroupPage = new PoliticalGroupPage(page);
  await politicalGroupPage.open();
  await politicalGroupPage.selectNumberOfSeats();
  await politicalGroupPage.fillRegisteredDesignation();
  await politicalGroupPage.fillStatutoryName();

  const authorisedAgentsPage = new AuthorisedAgentsPage(page);

   const authorisedAgent: AuthorisedAgent = {
      initials: "K",
      lastNamePrefix: "de",
      lastName: "Kraai",
    };
    const authorisedAgentTwo: AuthorisedAgent = {
      initials: "E",
      lastName: "Ekster",
    };
    await authorisedAgentsPage.addAuthorisedAgent([authorisedAgent, authorisedAgentTwo]);

}
)
