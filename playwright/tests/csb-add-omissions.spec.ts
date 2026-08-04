import { expect } from "@playwright/test";
import { test } from "./fixtures.ts";
import { csbCorrectionsPage } from "./pages/csbCorrectionsPage.ts";
import { CsbExaminationPage } from "./pages/csbExaminationPage.ts";
import { CsbGeneralInformationPage } from "./pages/csbGeneralInformationPage.ts";
import { CsbImportPage } from "./pages/csbImportPage.ts";
import { csbOmissionsPartyPage } from "./pages/csbOmissionsPartyPage.ts";
import { CsbOverviewPage } from "./pages/csbOverviewPage.ts";
import { csbPoliticalGroupPage } from "./pages/csbPoliticalGroupPage.ts";

test("check general information and add corrections and omissions", async ({ csbLogin }) => {
  const { page, groupName, lastEventHash } = csbLogin;
  const overviewPage = new CsbOverviewPage(page);
  const examinationPage = new CsbExaminationPage(page);
  const politicalGroupPage = new csbPoliticalGroupPage(page);
  const generalInformationPage = new CsbGeneralInformationPage(page);
  const correctionsPage = new csbCorrectionsPage(page);
  const importPage = new CsbImportPage(page);
  const omissionsPage = new csbOmissionsPartyPage(page);

  await expect(overviewPage.headerElection).toBeVisible();
  await overviewPage.linkExamination.click();

  await expect(examinationPage.headerExamination).toBeVisible();

  await examinationPage.linkAddPoliticalGroup.click();
  await expect(importPage.headerImport).toBeVisible();
  await importPage.textfieldHashcode.fill(lastEventHash);

  await Promise.all([
    page.waitForURL(/\/csb\/examination\/[^/]+/),
    page.getByRole("button", { name: "Importeren" }).click(),
  ]);

  await politicalGroupPage.selectedGroup(groupName);
  await politicalGroupPage.linkGeneralInformation.click();

  await expect(generalInformationPage.headerGeneralInformation).toBeVisible();
  await generalInformationPage.linkRegisteredDesignation.click();

  await correctionsPage.addCorrection("KDP");
  await expect(generalInformationPage.textCorrectedName).toHaveText("KDP");

  // Add each type of omission, verify and then remove
  const omissions = [
    {
      button: omissionsPage.buttonAuthoriseAppelation,
      text: "De machtiging aanduiding ontbreekt",
      resolvable: true,
    },
    {
      button: omissionsPage.buttonAuthorisedAgent,
      text: "De gemachtigde is niet geregistreerd",
      resolvable: true,
    },
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
