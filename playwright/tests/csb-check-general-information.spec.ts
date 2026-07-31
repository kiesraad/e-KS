import { expect, Page } from "@playwright/test";
import { test } from "./fixtures.ts";
import { CsbOverviewPage } from "./pages/csbOverviewPage.ts";
import { CsbExaminationPage } from "./pages/csbExaminationPage.ts";
import { csbPoliticalGroupPage } from "./pages/csbPoliticalGroupPage.ts";
import { CsbGeneralInformationPage } from "./pages/csbGeneralInformationPage.ts";
import { csbCorrectionsPage } from "./pages/csbCorrectionsPage.ts";

  test("check general information", async ({ loginCSB: page }) => {
    const overviewPage = new CsbOverviewPage(page);
    const examinationPage = new CsbExaminationPage(page);
    const politicalGroupPage = new csbPoliticalGroupPage(page);
    const generalInformationPage = new CsbGeneralInformationPage(page);
    const correctionsPage = new csbCorrectionsPage(page);

    await expect(overviewPage.headerElection).toBeVisible();
    await overviewPage.linkExamination.click();

    await expect(examinationPage.headerExamination).toBeVisible();
    await examinationPage.selectPoliticalGroup("Kiesraad Demo");

    await politicalGroupPage.selectedGroup("Kiesraad Demo");
    await politicalGroupPage.linkGeneralInformation.click();

    await expect(generalInformationPage.headerGeneralInformation).toBeVisible();
    await generalInformationPage.linkRegisteredDesignation.click();

    await correctionsPage.addCorrection("KDP");
    await expect(generalInformationPage.textCorrectedName).toHaveText("KDP");

  });