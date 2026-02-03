import { Page } from "@playwright/test";

export class AuthorisedAgentsPage {
  private readonly page: Page;

  constructor(page: Page) {
    this.page = page;
  }

  async open() {
    await this.page.goto("/political-group/authorised-agents");
  }

  async addAuthorisedAgent() {
    await this.page.getByRole("link", {name: "Gemachtigde toevoegen"}).click();
    await this.page.getByRole("textbox", { name: "Voorletters *" }).fill("A");
    await this.page.getByRole("textbox", { name: "Voorvoegsel" }).fill("de");
    await this.page.getByRole("textbox", { name: "Achternaam *" }).fill("Tester");
    await this.page.getByRole("button", {name: "Opslaan"}).click();
  }

  async editAuthorisedAgent() {
    await this.page.getByRole("cell", {name: "Gemachtigde toevoegen"}).click();
    await this.page.getByRole("textbox", { name: "Voorletters *" }).fill("A");
    await this.page.getByRole("textbox", { name: "Voorvoegsel" }).fill("de");
    await this.page.getByRole("textbox", { name: "Achternaam *" }).fill("Tester");
    await this.page.getByRole("button", {name: "Opslaan"}).click();
  }

    async removeAuthorisedAgent() {
    await this.page.getByRole("cell", {name: "Gemachtigde toevoegen"}).click();
    await this.page.getByRole("button", {name: "Gemachtigde verwijderen"}).click();
    await this.page.getByRole("button", {name: "Verwijderen", exact:true}).click();
  }
}