import type { Locator, Page } from "@playwright/test";

export class PoliticalGroupPage {
  readonly selectMoreThan16Seats: Locator;
  readonly selectLessThan16Seats: Locator;
  readonly textfieldRegisteredDesignation: Locator;
  readonly textfieldStatutoryName: Locator;
  readonly buttonSaveandNext: Locator;

  constructor(protected readonly page: Page) {
    this.buttonSaveandNext = this.page.getByRole("button", {
      name: "Opslaan en volgende",
    });
    this.selectMoreThan16Seats = this.page.getByRole("radio", { name: "Ja" });
    this.selectLessThan16Seats = this.page.getByRole("radio", { name: "Nee" });
    this.textfieldRegisteredDesignation = this.page.getByRole("textbox", {
      name: "Geregistreerde aanduiding",
    });
    this.textfieldStatutoryName = this.page.getByLabel(
      "Volledige statutaire naam",
    );
  }

  async open() {
    await this.page.goto("/political-group");
  }

  async save() {
    await this.page
      .getByRole("button", { name: "Opslaan en volgende" })
      .click();
  }

  /**
   * Selects whether a political group had 16 or more seats in previous election
   * @param input accepted values "Ja" or "Nee"
   */
  async selectHasMoreThan16Seats(input: string) {
    await this.page.getByRole("radio", { name: input }).check();
  }

  async setRegisteredDesignation(registeredDesignation: string) {
    await this.page
      .getByRole("textbox", { name: "Geregistreerde aanduiding" })
      .fill(registeredDesignation);
  }

  async setStatutoryName(statutoryName: string) {
    await this.page.getByLabel("Volledige statutaire naam").fill(statutoryName);
  }
}
