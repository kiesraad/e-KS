import { expect } from "@playwright/test";
import { test } from "./fixtures.ts";
import type { ListSubmitter } from "./models/listSubmitter.ts";
import type { NameAuthorisation } from "./models/nameAuthorisation.ts";
import { ListDesignationPage } from "./pages/listDesignationPage.ts";
import { ListSubmittersPage } from "./pages/listSubmittersPage.ts";
import { NameAuthorisationPage } from "./pages/nameAuthorisationPage.ts";
import { PoliticalGroupPage } from "./pages/politicalGroupPage.ts";
import { SubstituteSubmittersPage } from "./pages/substituteSubmittersPage.ts";
import { randomName } from "./utils/random.ts";
import { OverviewPage } from "./pages/overviewPage.ts";
//user logs in
test("full happy flow", async ({
    noExistingData: page,
  }) => {
//navgiate from home page
    const overviewPage = new OverviewPage(page);
    await overviewPage.linkGeneralInformation.click();
    await page.waitForURL("/political-group");
//fill in general information

    const listDesignationPage = new ListDesignationPage(page);
    await listDesignationPage.selectStandalone.check();
    await listDesignationPage.buttonSaveAndNext.click();
    await page.waitForURL("/political-group/information");

    const politicalGroupPage = new PoliticalGroupPage(page);
    await politicalGroupPage.selectNoSeats.check();
    await politicalGroupPage.textfieldRegisteredDesignation.fill("Test Partij");
    await politicalGroupPage.buttonSaveAndNext.click();
    await page.waitForURL("/political-group/name-authorisation");

    const authorisation: NameAuthorisation = {
        initials: "T",
        lastNamePrefix: "van",
        lastName: "Tester",
        legalName: "Kiesraad Test Partij",
    };
    const nameAuthorisationPage = new NameAuthorisationPage(page);
    await nameAuthorisationPage.addNameAuthorisation(authorisation);
    await nameAuthorisationPage.buttonNext.click();
    await page.waitForURL("/political-group/list-submitter");

    const submitter: ListSubmitter = {
        initials: "L",
        lastNamePrefix: "de",
        lastName: "Inleveraar",
        postalCode: "1234 AB",
        houseNumber: "1",
        houseNumberAddition: "a",
        streetName: "Teststraat",
        locality: "Teststad",
    };
    const listSubmittersPage = new ListSubmittersPage(page);
    await listSubmittersPage.setListSubmitter(submitter);

    const substitute: ListSubmitter = {
        initials: "V",
        lastNamePrefix: "ter",
        lastName: "Vervanger",
        postalCode: "5678 CD",
        houseNumber: "2",
        houseNumberAddition: "b",
        streetName: "Testlaan",
        locality: "Testdorp",
    };
    const substituteSubmittersPage = new SubstituteSubmittersPage(page);
    await substituteSubmittersPage.addSubstituteSubmitter(substitute);
    await listSubmittersPage.buttonNext.click();
    await page.waitForURL("/");

//create candidate list and add candidates

//submit list
//create list and add candidates

//submit list

  });
