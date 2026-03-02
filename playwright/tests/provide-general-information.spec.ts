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
    deleteExistingData,
  }) => {
    const politicalGroupPage = new PoliticalGroupPage(deleteExistingData);
    await politicalGroupPage.open();
    await politicalGroupPage.selectHasMoreThan16Seats("Ja");
    await politicalGroupPage.open();
    await politicalGroupPage.setRegisteredDesignation("TP");
    await politicalGroupPage.open();
    await politicalGroupPage.setStatutoryName("De Testpartij");
  });

  test("provide authorised agent", async ({ deleteExistingData }) => {
    await deleteExistingData.goto("/political-group/authorised-agents");
    const agent: AuthorisedAgent = {
      initials: "K",
      lastNamePrefix: "van",
      lastName: `Jansen ${randomName()}`,
    };
    const authorisedAgentsPage = new AuthorisedAgentsPage(deleteExistingData);
    await authorisedAgentsPage.addAuthorisedAgent(agent);

    const agentLastName = agent.lastNamePrefix
      ? `${agent.lastNamePrefix} ${agent.lastName}`
      : agent.lastName;

    await expect(
      authorisedAgentsPage.getAgentLocator(agentLastName),
    ).toBeVisible();
  });

  test("provide multiple list submitters", async ({ deleteExistingData }) => {
    await deleteExistingData.goto("/political-group/list-submitters");
    const submitterOne: ListSubmitter = {
      initials: "C",
      lastNamePrefix: "de",
      lastName: `Vries ${randomName()}`,
    };
    const submitterTwo: ListSubmitter = {
      initials: "Z",
      lastName: `Zeeman ${randomName()}`,
    };
    const listSubmittersPage = new ListSubmittersPage(deleteExistingData);

    for (const submitter of [submitterOne, submitterTwo]) {
      await listSubmittersPage.addListSubmitter(submitter);
    }
    for (const submitter of [submitterOne, submitterTwo]) {
      const submitterLastName = submitter.lastNamePrefix
        ? `${submitter.lastNamePrefix} ${submitter.lastName}`
        : submitter.lastName;
      await expect(
        listSubmittersPage.getSubmitterLocator(submitterLastName),
      ).toBeVisible();
    }
  });

  test("provide substitute list submitter", async ({ deleteExistingData }) => {
    await deleteExistingData.goto("/political-group/list-submitters");
    const submitterOne: ListSubmitter = {
      initials: "B",
      lastNamePrefix: "van",
      lastName: `Beers ${randomName()}`,
    };
    const submitterTwo: ListSubmitter = {
      initials: "O",
      lastName: `Smit ${randomName()}`,
    };
    const substituteSubmittersPage = new SubstituteSubmittersPage(
      deleteExistingData,
    );

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
