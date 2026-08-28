import { expect } from "@playwright/test";
import { test } from "./fixtures.ts";
import { CsbExaminationPage } from "./pages/csb/csbExaminationPage.ts";
import { CsbImportPage } from "./pages/csb/csbImportPage.ts";
import { CsbOverviewPage } from "./pages/csb/csbOverviewPage.ts";

test("import a political group in the CSB dashboard by hash", async ({
  csbLogin,
}) => {
  const { page, groupName, lastEventHash } = csbLogin;
  expect(lastEventHash).not.toBe("");

  const overviewPage = new CsbOverviewPage(page);
  const examinationPage = new CsbExaminationPage(page);
  const importPage = new CsbImportPage(page);

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

  await expect(page.getByRole("heading", { name: groupName })).toBeVisible();
});
