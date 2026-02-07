import type { Page } from "@playwright/test";

export class SelectElectoralDistrictsPage {
  private readonly page: Page;

  constructor(page: Page) {
    this.page = page;
  }
  async selectDistricts(districts: string[]) {
    for (const district of districts) {
      await this.page.getByRole("checkbox", { name: district }).check();
    }

    await this.page.getByRole("button", { name: "Opslaan en verder" }).click();
    // TODO Grietje: deze test fatsoenlijk fixen met selecteren list submitter
    await this.page.getByRole("link", { name: "Sluiten" }).click();
  }
}
