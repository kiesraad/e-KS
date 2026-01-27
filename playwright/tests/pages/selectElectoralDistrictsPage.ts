import type { Page } from "@playwright/test";

export class SelectElectoralDistrictsPage {
  private readonly page: Page;

  constructor(page: Page) {
    this.page = page;
  }
  async selectDistricts(districts: string[]) {
    for (var district of districts) {
      await this.page.getByRole("checkbox", { name: district }).check();
    }
    await this.page.getByRole("button", { name: "Save" }).click();
  }
}
