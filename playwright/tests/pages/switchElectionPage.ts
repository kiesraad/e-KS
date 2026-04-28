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
    this.dropdownElections = this.page.getByLabel("Verkiezing wisselen");
    this.dropdownProvinces = this.page.getByLabel("Provincie");
    this.dropdownWaterAuthorities = this.page.getByLabel("Waterschap");
    this.buttonSwitch = this.page.getByRole("button", {
      name: "Verkiezing wisselen",
    });
  }

  async selectedElection(election: string) {
    await expect(this.page.getByRole("cell", { name: election })).toBeVisible();
  }
}
