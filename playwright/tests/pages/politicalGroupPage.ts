import type { Page } from "@playwright/test";

export class PoliticalGroupPage {
  private readonly page: Page;

  constructor(page: Page) {
    this.page = page;
  }

  async open() {
    await this.page.goto("/political-group");
  }

  /**
   * Selects whether a political group had 16 or more seats in previous election
   * @param input accepted values "Ja" or "Nee"
   */
  async selectHasMoreThan16Seats(input: string) {
    await this.page.getByRole("radio", { name: input }).check();
    await this.page.getByRole("button", { name: "Opslaan" }).click();
  }

  async setRegisteredDesignation(registeredDesignation: string) {
    await this.page
      .getByRole("textbox", { name: "Geregistreerde aanduiding" })
      .fill(registeredDesignation);
    await this.page.getByRole("button", { name: "Opslaan" }).click();
  }

  async setStatutoryName(statutoryName: string) {
    await this.page.getByLabel("Volledige statutaire naam").fill(statutoryName);
    await this.page.getByRole("button", { name: "Opslaan" }).click();
  }
}
