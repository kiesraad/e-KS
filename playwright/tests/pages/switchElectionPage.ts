import { expect, type Locator, type Page } from "@playwright/test";

export class SwitchElectionPage {
  readonly headerSwitchElection: Locator;
  readonly dropdownElections: Locator;
  readonly dropdownProvinces: Locator;
  readonly dropdownWaterAuthorities: Locator;
  readonly buttonSwitch: Locator;   


  constructor(protected readonly page: Page) {
    this.headerSwitchElection = this.page.getByRole("heading", {
      name: "Verkiezing wisselen",
    });
    this.dropdownElections = this.page.getByLabel('Verkiezing wisselen');
    this.dropdownProvinces = this.page.getByLabel('Provincie');
    this.dropdownWaterAuthorities = this.page.getByLabel('Waterschap');
    this.buttonSwitch = this.page.getByRole("button", {
      name: "Verkiezing wisselen",
    });


  }
async selectedElection(election: string) {
await expect(this.page.getByRole('cell', { name: election })).toBeVisible();
  }

  async verifyElectionExists(election: string, region: string) {
    let tableRows = await this.page.locator('table tbody tr').all();
    console.log(await tableRows.length);
    let row = tableRows.filter({ has: this.page.getByRole('cell', { name: election }) });
    console.log(await row.count());
    row = row.filter({ has: this.page.getByRole('cell', { name: region }) });
    await expect(row).toBeVisible();
    await expect(row).toHaveCount(1);
  }
  }