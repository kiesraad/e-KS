import { Page } from "@playwright/test";

export class ListSumbittersPage {
  private readonly page: Page;

  constructor(page: Page) {
    this.page = page;
  }

  async open() {
    await this.page.goto("/political-group/list-submitters");
  }

  async addListSubmitter() {
    await this.page.getByRole("link", {name: "Lijstinleveraar toevoegen"}).click();
    await this.page.getByRole("textbox", { name: "Voorletters *" }).fill("A");
    await this.page.getByRole("textbox", { name: "Voorvoegsel" }).fill("de");
    await this.page.getByRole("textbox", { name: "Achternaam *" }).fill("Tester");
    await this.page.getByRole("button", {name: "Opslaan"}).click();
  }
    async editListSubmitter() {
    await this.page.getByRole("cell", {name: "Gemachtigde toevoegen"}).click();
    await this.page.getByRole("textbox", { name: "Voorletters *" }).fill("A");
    await this.page.getByRole("textbox", { name: "Voorvoegsel" }).fill("de");
    await this.page.getByRole("textbox", { name: "Achternaam *" }).fill("Tester");
    await this.page.getByRole("button", {name: "Opslaan"}).click();
  }

    async removeListSubmitter() {
    await this.page.getByRole("cell", {name: "Gemachtigde toevoegen"}).click();
    await this.page.getByRole("button", {name: "Lijstinleveraar verwijderen"}).click();
    await this.page.getByRole("button", {name: "Verwijderen", exact:true}).click();
  }
}