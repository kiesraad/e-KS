import { Page } from "@playwright/test";
import type { PoliticalGroup } from "../models/politicalGroup";

export class PoliticalGroupPage {
  private readonly page: Page;

  constructor(page: Page) {
    this.page = page;
  }

  async open() {
    await this.page.goto("/political-group");
  }

  async selectNumberOfSeats() {
    await this.page.getByRole("radio", {name: "Ja"} ).check();
    await this.page.getByRole("button", { name: "Opslaan" }).click();
  }

  async fillRegisteredDesignation() {
    await this.page.getByLabel("Geregistreerde aanduiding").fill("TP");
    await this.page.getByRole("button", { name: "Opslaan" }).click();
  }

  async fillStatutoryName() {
    await this.page.getByLabel("Volledige statutaire naam").fill("De Testpartij");
    await this.page.getByRole("button", { name: "Opslaan" }).click();
  }
}

