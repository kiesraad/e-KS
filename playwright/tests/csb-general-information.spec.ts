import { expect } from "@playwright/test";
import { test } from "./fixtures.ts";
import type { NameAuthorisation } from "./models/nameAuthorisation.ts";
import { csbCorrectionsPage } from "./pages/csb/csbCorrectionsPage.ts";
import { CsbGeneralInformationPage } from "./pages/csb/csbGeneralInformationPage.ts";
import { csbOmissionsPartyPage } from "./pages/csb/csbOmissionsPartyPage.ts";
import { csbPoliticalGroupPage } from "./pages/csb/csbPoliticalGroupPage.ts";
import { ListDesignationPage } from "./pages/pp/listDesignationPage.ts";
import { NameAuthorisationPage } from "./pages/pp/nameAuthorisationPage.ts";
import { OverviewPage } from "./pages/pp/overviewPage.ts";
import { PoliticalGroupPage } from "./pages/pp/politicalGroupPage.ts";

test.describe("check general information and add corrections and omissions", async () => {
  test("for standalone political group", async ({ csbImport }) => {
    const { page, groupName } = csbImport;
    const politicalGroupPage = new csbPoliticalGroupPage(page);
    const generalInformationPage = new CsbGeneralInformationPage(page);
    const correctionsPage = new csbCorrectionsPage(page);
    const omissionsPage = new csbOmissionsPartyPage(page);

    await politicalGroupPage.selectedGroup(groupName);
    await politicalGroupPage.linkGeneralInformation.click();

    await expect(generalInformationPage.headerGeneralInformation).toBeVisible();
    await generalInformationPage.linkRegisteredDesignation.click();

    await correctionsPage.addCorrection("KDP");
    await expect(generalInformationPage.textCorrectedName).toHaveText("KDP");

    // Add each type of omission, verify and then remove
    const omissions = [
      {
        button: omissionsPage.buttonRegisterAppelation,
        text: "De aanduiding is niet geregistreerd",
        resolvable: false,
      },
    ];

    for (const { button, text, resolvable } of omissions) {
      await generalInformationPage.linkAddOmission.click();
      await expect(
        page.getByRole("heading", { name: "Verzuimen - Basisgegevens (KDP)" }),
      ).toBeVisible();
      await button.click();
      if (resolvable) {
        await expect(omissionsPage.checkboxRecoverable).toBeChecked();
        await omissionsPage.textfieldLetter.fill("Testtoevoeging");
      }
      await omissionsPage.buttonAddAndClose.click();
      await generalInformationPage.linkManageOmissions.click();
      await expect(page.getByText(text)).toBeVisible();
      if (resolvable) {
        await expect(page.getByText("Testtoevoeging")).toBeVisible();
        await expect(page.getByText("Herstelbaar")).toBeVisible();
      } else {
        await expect(
          page.getByText("Onherstelbaar", { exact: true }),
        ).toBeVisible();
      }
      await omissionsPage.buttonRemoveOmission.click();
      await omissionsPage.linkClose.click();
    }
  });

  test("for combination", async ({ csbImport }) => {
    const { page, groupName } = csbImport;
    const politicalGroupPage = new csbPoliticalGroupPage(page);
    const generalInformationPage = new CsbGeneralInformationPage(page);
    const omissionsPage = new csbOmissionsPartyPage(page);

    await politicalGroupPage.selectedGroup(groupName);

    // add paper correction to select combination
    await politicalGroupPage.buttonPaperCorrections.click();
    const pcOverviewPage = new OverviewPage(page);
    await pcOverviewPage.linkGeneralInformation.click();
    const listDesignationPage = new ListDesignationPage(page);
    await page.goto("/political-group");
    await listDesignationPage.selectCombined.check();
    await listDesignationPage.buttonSaveAndNext.click();
    await page.waitForURL("/political-group/information");

    const pcPoliticalGroupPage = new PoliticalGroupPage(page);
    await pcPoliticalGroupPage.selectMoreThan16Seats.check();
    await pcPoliticalGroupPage.textfieldCombinedDesignation.fill("TP/TP2");
    await pcPoliticalGroupPage.buttonSaveAndNext.click();
    await page.waitForURL("/political-group/name-authorisation");

    await page.goto("/political-group/information");
    await expect(pcPoliticalGroupPage.selectMoreThan16Seats).toBeChecked();
    await expect(pcPoliticalGroupPage.textfieldCombinedDesignation).toHaveValue(
      "TP/TP2",
    );
    await pcPoliticalGroupPage.buttonSaveAndNext.click();

    const authorisationOne: NameAuthorisation = {
      initials: "K.J.",
      lastNamePrefix: "van",
      lastName: "Veen",
      legalName: "Eerste Partij",
    };

    const authorisationTwo: NameAuthorisation = {
      initials: "D.F.",
      lastNamePrefix: "de",
      lastName: "Boer",
      legalName: "Tweede Partij",
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
    await page.getByRole("button", { name: "Terug naar onderzoek" }).click();

    // exit paper corrections mode and verify changes
    await politicalGroupPage.linkGeneralInformation.click();
    await expect(generalInformationPage.headerGeneralInformation).toBeVisible();
    await expect(generalInformationPage.textCorrectedName).toHaveText("TP/TP2");
    await expect(generalInformationPage.textCorrectedType).toHaveText(
      "Combinatie van meerdere geregistreerde namen",
    );
    for (const text of [
      "Eerste Partij",
      "Tweede Partij",
      "K.J.",
      "D.F.",
      "van Veen",
      "de Boer",
    ]) {
      await expect(page.getByText(text).first()).toBeVisible();
    }

    // Add each type of omission, verify and then remove
    const omissions = [
      {
        button: omissionsPage.buttonRegisterCombination,
        text: "De aanduiding(en) is/zijn niet geregistreerd",
        resolvable: false,
      },
    ];

    for (const { button, text, resolvable } of omissions) {
      await generalInformationPage.linkAddOmission.click();
      await expect(
        page.getByRole("heading", {
          name: "Verzuimen - Basisgegevens (TP/TP2)",
        }),
      ).toBeVisible();
      await button.click();
      if (resolvable) {
        await expect(omissionsPage.checkboxRecoverable).toBeChecked();
        await omissionsPage.textfieldLetter.fill("Testtoevoeging");
      }
      await omissionsPage.buttonAddAndClose.click();
      await generalInformationPage.linkManageOmissions.click();
      await expect(page.getByText(text)).toBeVisible();
      if (resolvable) {
        await expect(page.getByText("Testtoevoeging")).toBeVisible();
        await expect(page.getByText("Herstelbaar")).toBeVisible();
      } else {
        await expect(
          page.getByText("Onherstelbaar", { exact: true }),
        ).toBeVisible();
      }
      await omissionsPage.buttonRemoveOmission.click();
      await omissionsPage.linkClose.click();
    }
  });
});
