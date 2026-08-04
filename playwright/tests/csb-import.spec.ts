import { expect } from "@playwright/test";
import { test } from "./fixtures.ts";

test("import a political group in the CSB dashboard by hash", async ({
  csbLogin,
}) => {
  const { page, groupName, lastEventHash } = csbLogin;
  expect(lastEventHash).not.toBe("");

  await page.goto("/csb/import");
  await page
    .getByLabel("Voer het begin van de hash code in")
    .fill(lastEventHash);

  await Promise.all([
    page.waitForURL(/\/csb\/examination\/[^/]+/),
    page.getByRole("button", { name: "Importeren" }).click(),
  ]);

  await expect(page.getByRole("heading", { name: groupName })).toBeVisible();
});
