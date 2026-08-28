import type { Locator, Page } from "@playwright/test";

export class SelectElectionPage {
  readonly HeaderElections: Locator;
  readonly dropdownElections: Locator;
  readonly dropdownProvinces: Locator;
  readonly dropdownWaterAuthorities: Locator;
  readonly buttonContinue: Locator;
  readonly checkboxFixtures: Locator;

  constructor(protected readonly page: Page) {
    this.HeaderElections = this.page.getByRole("heading", {
      name: "Kiesraad - Kandidaatstelling",
    });
    this.dropdownElections = this.page.getByLabel("Verkiezing", {
      exact: true,
    });
    this.dropdownProvinces = this.page.getByLabel("Provincie");
    this.dropdownWaterAuthorities = this.page.getByLabel("Waterschap");
    this.buttonContinue = this.page.getByRole("button", {
      name: "Verder",
    });
    this.checkboxFixtures = this.page.getByLabel(
      "Voorbeelddata laden voor deze verkiezing",
    );
  }
}
