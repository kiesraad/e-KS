import { expect } from "@playwright/test";
import { test } from "./fixtures.ts";
import { SwitchElectionPage } from "./pages/switchElectionPage.ts";


test.describe("switch election", async () => {
  test("provincial council", async ({ login: page }) => {
    await page.goto(`/dev/login?fixtures=true`);
    await page.goto("/switch-election");
    const switchElectionPage = new SwitchElectionPage(page);
    await expect(switchElectionPage.headerSwitchElection).toBeVisible();
    await expect(switchElectionPage.selectedElection("Eerste Kamerverkiezing der")).toBeVisible();
    await switchElectionPage.dropdownElections.selectOption("Provinciale Statenverkiezingen 2027");
    await switchElectionPage.dropdownProvinces.selectOption("Limburg");
    await switchElectionPage.buttonSwitch.click();
  });

  test("water authority", async ({ login: page }) => {
    await page.goto(`/dev/login?fixtures=true`);
    await page.goto("/switch-election");
    const switchElectionPage = new SwitchElectionPage(page);
    await expect(switchElectionPage.headerSwitchElection).toBeVisible();
    await switchElectionPage.dropdownElections.selectOption("Waterschapverkiezingen 2027");
    await switchElectionPage.dropdownWaterAuthorities.selectOption("Hunze en Aa's");
    await switchElectionPage.buttonSwitch.click();
  });
});

