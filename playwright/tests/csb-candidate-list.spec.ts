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

test.describe("check candidate list and add corrections and omissions", async () => {
  test("for standalone political group", async ({ csbImport }) => {
    const { page, groupName } = csbImport;
    const politicalGroupPage = new csbPoliticalGroupPage(page);
    const generalInformationPage = new CsbGeneralInformationPage(page);
    const correctionsPage = new csbCorrectionsPage(page);
    const omissionsPage = new csbOmissionsPartyPage(page);

    await politicalGroupPage.selectedGroup(groupName);
    await politicalGroupPage.linkCandidateList.first().click();

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