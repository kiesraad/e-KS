import { expect } from "@playwright/test";
import { test } from "./fixtures.ts";
import { CandidateListsOverviewPage } from "./pages/pg/candidateListsOverviewPage.ts";
import { EditListDetailsPage } from "./pages/pg/editListDetailsPage.ts";
import { ManageCandidateListPage } from "./pages/pg/manageCandidateListPage.ts";
import { OverviewPage } from "./pages/pg/overviewPage.ts";
import { SwitchElectionPage } from "./pages/pg/switchElectionPage.ts";

test.describe("switch election", () => {
  test.beforeEach(
    "navigate to switch election page",
    async ({ login: page }) => {
      await page.goto("/switch-election");
      const switchElectionPage = new SwitchElectionPage(page);
      await expect(switchElectionPage.headerSwitchElection).toBeVisible();
      await switchElectionPage.selectedElection("Eerste Kamerverkiezing der");
    },
  );

  test("provincial council", async ({ login: page }) => {
    const switchElectionPage = new SwitchElectionPage(page);
    await switchElectionPage.dropdownElections.selectOption(
      "Provinciale Statenverkiezingen 2027",
    );
    await switchElectionPage.dropdownProvinces.selectOption("Limburg");
    await switchElectionPage.buttonSwitch.click();

    const overviewPage = new OverviewPage(page);
    await overviewPage.selectedElection(
      "Provinciale Statenverkiezingen 2027 - Limburg",
    );
    await overviewPage.linkCandidateList.click();

    await new CandidateListsOverviewPage(page).buttonAddList.click();

    await new EditListDetailsPage(page).addDistricts(["Venlo"]);

    await expect(
      page.locator("//li/a[normalize-space()='2. Venlo']"),
    ).toBeVisible();
  });

  test("water authority", async ({ login: page }) => {
    const switchElectionPage = new SwitchElectionPage(page);
    await switchElectionPage.dropdownElections.selectOption(
      "Waterschapsverkiezingen 2027",
    );
    await switchElectionPage.dropdownWaterAuthorities.selectOption(
      "Hunze en Aa's",
    );
    await switchElectionPage.buttonSwitch.click();

    const overviewPage = new OverviewPage(page);
    await overviewPage.selectedElection(
      "Waterschapsverkiezingen 2027 - Hunze en Aa's",
    );
    await overviewPage.linkCandidateList.click();

    await new CandidateListsOverviewPage(page).buttonAddList.click();

    await expect(new ManageCandidateListPage(page).buttonCSV).toBeVisible();
  });
});
