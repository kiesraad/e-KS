import { expect } from "@playwright/test";
import { test } from "./fixtures.ts";
import type { NameAuthorisation } from "./models/nameAuthorisation.ts";
import type { ListSubmitter } from "./models/listSubmitter.ts";
import { NameAuthorisationPage } from "./pages/nameAuthorisationPage.ts";
import { ListDesignationPage } from "./pages/listDesignationPage.ts";
import { ListSubmittersPage } from "./pages/listSubmittersPage.ts";
import { PoliticalGroupPage } from "./pages/politicalGroupPage.ts";
import { SubstituteSubmittersPage } from "./pages/substituteSubmittersPage.ts";
import { randomName } from "./utils/random.ts";

test.describe("provide general information for political group", async () => {
  test("provide general information for political group", async ({
    noExistingData: page,
  }) => {
    const listDesignationPage = new ListDesignationPage(page);
    await page.goto("/political-group");
    await listDesignationPage.selectStandalone.check();
    await listDesignationPage.buttonSaveAndNext.click();
    await page.waitForURL("/political-group/information");

    const politicalGroupPage = new PoliticalGroupPage(page);
    await politicalGroupPage.selectMoreThan16Seats.check();
    await politicalGroupPage.textfieldRegisteredDesignation.fill("TP");
    await politicalGroupPage.buttonSaveAndNext.click();
    await page.waitForURL("/political-group/authorised-agents");

    await page.goto("/political-group/information");
    await expect(politicalGroupPage.selectMoreThan16Seats).toBeChecked();
    await expect(politicalGroupPage.textfieldRegisteredDesignation).toHaveValue(
      "TP",
    );
  });

  test("provide authorised agent", async ({ noExistingData: page }) => {
    await page.goto("/political-group/authorised-agents");

    const authorisation: NameAuthorisation = {
      initials: "K",
      lastNamePrefix: "van",
      lastName: `Jansen ${randomName()}`,
      legalName: "Kiesraad Demo Partij",
    };
    const nameAuthorisationPage = new NameAuthorisationPage(page);
    await nameAuthorisationPage.addNameAuthorisation(authorisation);

    const agentLastName = authorisation.lastNamePrefix
      ? `${authorisation.lastNamePrefix} ${authorisation.lastName}`
      : authorisation.lastName;

    await expect(
      nameAuthorisationPage.getAgentLocator(agentLastName),
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
