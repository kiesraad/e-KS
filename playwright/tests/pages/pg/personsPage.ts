import type { Locator, Page } from "@playwright/test";

export class PersonsPage {
  private readonly page: Page;
  readonly linkAddPerson: Locator;
  constructor(page: Page) {
    this.page = page;
    this.linkAddPerson = page.getByRole("link", {
      name: "Kandidaat toevoegen",
    });
  }

  async getCellLastName(lastName: string) {
    return this.page.getByRole("cell", { name: lastName });
  }
}
