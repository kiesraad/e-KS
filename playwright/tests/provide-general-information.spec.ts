import { expect } from "@playwright/test";
import { test } from "./fixtures.ts";
import type { ListSubmitter } from "./models/listSubmitter.ts";
import type { NameAuthorisation } from "./models/nameAuthorisation.ts";
import { CandidateListsOverviewPage } from "./pages/pg/candidateListsOverviewPage.ts";
import { ListDesignationPage } from "./pages/pg/listDesignationPage.ts";
import { ListSubmittersPage } from "./pages/pg/listSubmittersPage.ts";
import { NameAuthorisationPage } from "./pages/pg/nameAuthorisationPage.ts";
import { PoliticalGroupPage } from "./pages/pg/politicalGroupPage.ts";
import { SubstituteSubmittersPage } from "./pages/pg/substituteSubmittersPage.ts";
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
    await page.waitForURL("/political-group/name-authorisation");

    await page.goto("/political-group/information");
    await expect(politicalGroupPage.selectMoreThan16Seats).toBeChecked();
    await expect(politicalGroupPage.textfieldRegisteredDesignation).toHaveValue(
      "TP",
    );
  });

  test("provide authorised agent", async ({ noExistingData: page }) => {
    await page.goto("/political-group/name-authorisation");

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

  test("provide information for combination", async ({
    noExistingData: page,
  }) => {
    const listDesignationPage = new ListDesignationPage(page);
    await page.goto("/political-group");
    await listDesignationPage.selectCombined.check();
    await listDesignationPage.buttonSaveAndNext.click();
    await page.waitForURL("/political-group/information");

    const politicalGroupPage = new PoliticalGroupPage(page);
    await politicalGroupPage.selectMoreThan16Seats.check();
    await politicalGroupPage.textfieldCombinedDesignation.fill("TP/TP2");
    await politicalGroupPage.buttonSaveAndNext.click();
    await page.waitForURL("/political-group/name-authorisation");

    await page.goto("/political-group/information");
    await expect(politicalGroupPage.selectMoreThan16Seats).toBeChecked();
    await expect(politicalGroupPage.textfieldCombinedDesignation).toHaveValue(
      "TP/TP2",
    );
    await politicalGroupPage.buttonSaveAndNext.click();

    const authorisationOne: NameAuthorisation = {
      initials: "K",
      lastNamePrefix: "van",
      lastName: "Jansen",
      legalName: "Test Partij 1",
    };

    const authorisationTwo: NameAuthorisation = {
      initials: "D",
      lastNamePrefix: "de",
      lastName: "Boer",
      legalName: "Test Partij 2",
    };
    const nameAuthorisationPage = new NameAuthorisationPage(page);

    for (const authorisation of [authorisationOne, authorisationTwo]) {
      await nameAuthorisationPage.addNameAuthorisation(authorisation);
    }

    for (const authorisation of [authorisationOne, authorisationTwo]) {
      const agentLastName = authorisation.lastNamePrefix
        ? `${authorisation.lastNamePrefix} ${authorisation.lastName}`
        : authorisation.lastName;
      await expect(
        nameAuthorisationPage.getAgentLocator(agentLastName),
      ).toBeVisible();
    }
  });

  test("provide general information for blank list", async ({
    noExistingData: page,
  }) => {
    const listDesignationPage = new ListDesignationPage(page);
    await page.goto("/political-group");
    await listDesignationPage.selectBlank.check();
    await listDesignationPage.buttonSaveAndNext.click();
    await page.waitForURL("/political-group/list-submitter");

    const submitter: ListSubmitter = {
      initials: "G",
      lastNamePrefix: "van",
      lastName: "Veen",
    };
    const listSubmittersPage = new ListSubmittersPage(page);

    await listSubmittersPage.setListSubmitter(submitter);

    const submitterLastName = submitter.lastNamePrefix
      ? `${submitter.lastNamePrefix} ${submitter.lastName}`
      : submitter.lastName;
    await expect(
      listSubmittersPage.getSubmitterLocator(submitterLastName),
    ).toBeVisible();
    await listSubmittersPage.buttonNext.click();
    await page.waitForURL("/");
    await expect(
      page.getByRole("heading", {
        name: "Eerste Kamerverkiezing der Staten-Generaal 2027",
      }),
    ).toBeVisible();
  });

  test("provide general information for party with less than 16 seats", async ({
    login: page,
  }) => {
    const listDesignationPage = new ListDesignationPage(page);
    await page.goto("/political-group");
    await listDesignationPage.selectStandalone.check();
    await listDesignationPage.buttonSaveAndNext.click();
    await page.waitForURL("/political-group/information");

    const politicalGroupPage = new PoliticalGroupPage(page);
    await politicalGroupPage.selectLessThan16Seats.check();
    await politicalGroupPage.buttonSaveAndNext.click();
    await page.waitForURL("/political-group/name-authorisation");

    await page.goto("/political-group/information");
    await expect(politicalGroupPage.selectLessThan16Seats).toBeChecked();
    await politicalGroupPage.linkCandidateLists.click();
    const candidateListsOverviewPage = new CandidateListsOverviewPage(page);
    await expect(page.getByText("55 / 50 kandidaten").first()).toBeVisible();
    await candidateListsOverviewPage.linkFinalize.click();
    await expect(page.getByText("5 kandidaten te veel").first()).toBeVisible();
  });
});
