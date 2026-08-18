import { expect } from "@playwright/test";
import { test } from "./fixtures.ts";
import { CsbExaminationPage } from "./pages/csb/csbExaminationPage.ts";
import { csbPoliticalGroupPage } from "./pages/csb/csbPoliticalGroupPage.ts";

test("finalize examination happy flow", async ({
  csbImport,
}) => {
  const { page, groupName } = csbImport;
  const examinationPage = new CsbExaminationPage(page);
  const politicalGroupPage = new csbPoliticalGroupPage(page);

await page.goto("/csb/examination");
await expect(page.getByText("Controle bezig")).toBeVisible();
await examinationPage.selectPoliticalGroup(groupName);
await expect(page.getByText("Onderzoek afronden")).toBeVisible();
await expect(page.locator(".candidate-lists")).not.toHaveClass(/disabled/);
await politicalGroupPage.switchFinalize.click();
await expect(page.getByText("Onderzoek afgerond")).toBeVisible();
await expect(page.locator(".candidate-lists")).toHaveClass(/disabled/);
await politicalGroupPage.buttonBack.click();
await expect(page.getByText("Goedgekeurd")).toBeVisible();
});
