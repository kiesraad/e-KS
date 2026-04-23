import { expect } from "@playwright/test";
import { test } from "./fixtures.ts";
import type { AuthorisedAgent } from "./models/authorisedAgent.ts";
import type { ListSubmitter } from "./models/listSubmitter.ts";
import { AuthorisedAgentsPage } from "./pages/authorisedAgentsPage.ts";
import { ListSubmittersPage } from "./pages/listSubmittersPage.ts";
import { PoliticalGroupPage } from "./pages/politicalGroupPage.ts";
import { SubstituteSubmittersPage } from "./pages/substituteSubmittersPage.ts";
import { randomName } from "./utils/random.ts";

test.describe("provide general information for political group", async () => {
  test("provide general information for political group", async ({
    noExistingData: page,
  }) => {
    const politicalGroupPage = new PoliticalGroupPage(page);
    await page.goto("/political-group");
    await politicalGroupPage.selectMoreThan16Seats.check();
    await politicalGroupPage.textfieldRegisteredDesignation.fill("TP");
    await politicalGroupPage.textfieldStatutoryName.fill("De Testpartij");
    await politicalGroupPage.buttonSaveandNext.click();
    await page.waitForURL("/political-group/authorised-agents");
    await page.goto("/political-group");

    await expect(politicalGroupPage.selectMoreThan16Seats).toBeChecked();
    await expect(politicalGroupPage.textfieldRegisteredDesignation).toHaveValue(
      "TP",
    );
    await expect(politicalGroupPage.textfieldStatutoryName).toHaveValue(
      "De Testpartij",
    );
  });

  test("provide authorised agent", async ({ noExistingData: page }) => {
    await page.goto("/political-group/authorised-agents");

    const agent: AuthorisedAgent = {
      initials: "K",
      lastNamePrefix: "van",
      lastName: `Jansen ${randomName()}`,
    };
    const authorisedAgentsPage = new AuthorisedAgentsPage(page);
    await authorisedAgentsPage.addAuthorisedAgent(agent);

    const agentLastName = agent.lastNamePrefix
      ? `${agent.lastNamePrefix} ${agent.lastName}`
      : agent.lastName;

    await expect(
      authorisedAgentsPage.getAgentLocator(agentLastName),
    ).toBeVisible();
  });

  test("provide list submitter", async ({ noExistingData: page }) => {
    await page.goto("/political-group/list-submitter");

    const submitter: ListSubmitter = {
      initials: "C",
      lastNamePrefix: "de",
      lastName: `Vries ${randomName()}`,
    };
    const listSubmittersPage = new ListSubmittersPage(page);

    await listSubmittersPage.setListSubmitter(submitter);

    const submitterLastName = submitter.lastNamePrefix
      ? `${submitter.lastNamePrefix} ${submitter.lastName}`
      : submitter.lastName;
    await expect(
      listSubmittersPage.getSubmitterLocator(submitterLastName),
    ).toBeVisible();
  });

  test("provide substitute list submitter", async ({
    noExistingData: page,
  }) => {
    await page.goto("/political-group/list-submitter");
    const submitterOne: ListSubmitter = {
      initials: "B",
      lastNamePrefix: "van",
      lastName: `Beers ${randomName()}`,
    };
    const submitterTwo: ListSubmitter = {
      initials: "O",
      lastName: `Smit ${randomName()}`,
    };
    const substituteSubmittersPage = new SubstituteSubmittersPage(page);

    for (const submitter of [submitterOne, submitterTwo]) {
      await substituteSubmittersPage.addSubstituteSubmitter(submitter);
    }

    for (const submitter of [submitterOne, submitterTwo]) {
      const submitterLastName = submitter.lastNamePrefix
        ? `${submitter.lastNamePrefix} ${submitter.lastName}`
        : submitter.lastName;
      await expect(
        substituteSubmittersPage.getSubmitterLocator(submitterLastName),
      ).toBeVisible();
    }
  });
});
