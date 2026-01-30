import { expect, type Page } from "@playwright/test";

export class CandidateListsOverviewPage {
  private readonly page: Page;

  constructor(page: Page) {
    this.page = page;
  }

  async open() {
    await this.page.goto("/candidate-lists");
  }

  async addList() {
    await this.page
      .getByRole("main")
      .getByRole("link", { name: "Lijst aanmaken" })
      .click();
  }

  async manageList() {
    await this.page
      .getByRole("link", { name: "Kandidatenlijst Kieskringen" })
      .first()
      .click();
  }

  async managePersons() {
    await this.page.getByRole("heading", { name: "Alle personen" }).click();
  }

  async checkRemovedDistricts(districts: string[]) {
    for (const district of districts) {
      await expect(
        this.page.getByRole("listitem", { name: district }),
      ).toHaveCount(0);
    }
  }
}
