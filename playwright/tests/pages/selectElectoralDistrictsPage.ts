import type { Locator, Page } from "@playwright/test";

export class SelectElectoralDistrictsPage {
  readonly buttonNext: Locator;
  readonly buttonClose: Locator;
  readonly buttonAdd: Locator;

  constructor(protected readonly page: Page) {
    this.buttonNext = this.page.getByRole("button", { name: "Volgende" });
    this.buttonClose = this.page.getByRole("link", { name: "Sluiten" }).first();
    this.buttonAdd = this.page.getByRole("button", { name: "Toevoegen" });
  }

  async selectDistricts(districts: string[]) {
    for (const district of districts) {
      await this.page.getByRole("checkbox", { name: district }).check();
    }

    await this.buttonNext.click();
    await this.buttonAdd.click();
  }
}
