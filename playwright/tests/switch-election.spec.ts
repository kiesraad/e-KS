import { expect } from "@playwright/test";
import { test } from "./fixtures.ts";


test.describe("switch election", async () => {
  test("provincial council", async ({ login: page }) => {
    await page.goto(`/dev/login?fixtures=true`);
    await page.goto("/switch-election");

  });

  test("water authority", async ({ login: page }) => {
    await page.goto(`/dev/login?fixtures=true`);
    await page.goto("/switch-election");

  });
});