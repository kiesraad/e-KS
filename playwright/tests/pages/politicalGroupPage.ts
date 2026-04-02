import type { Page } from "@playwright/test";

export class PoliticalGroupPage {
  private readonly page: Page;

  constructor(page: Page) {
    this.page = page;
  }

  async open() {
    await this.page.goto("/political-group");
  }

  async save() {
    await this.page.getByRole("button", { name: "Opslaan en volgende" }).click();
  }

  /**
   * Selects whether a political group had 16 or more seats in previous election
   * @param input accepted values "Ja" or "Nee"
   */
  async selectHasMoreThan16Seats(input: string) {
    await this.page.getByRole("radio", { name: input }).check();
    await this.save();
  }

  async setRegisteredDesignation(registeredDesignation: string) {
    await this.page
      .getByRole("textbox", { name: "Geregistreerde aanduiding" })
      .fill(registeredDesignation);
    await this.save();
  }

  async setStatutoryName(statutoryName: string) {
    await this.page.getByLabel("Volledige statutaire naam").fill(statutoryName);
    await this.save();
  }
}
