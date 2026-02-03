import { Page } from "@playwright/test";
import type { AuthorisedAgent } from "../models/authorisedAgent";

export class AuthorisedAgentsPage {
  private readonly page: Page;

  constructor(page: Page) {
    this.page = page;
  }

  async open() {
    await this.page.goto("/political-group/authorised-agents");
  }

  async addAuthorisedAgent(authorisedAgents: AuthorisedAgent[]) {
      for (const authorisedAgent of authorisedAgents) {
    await this.page.getByRole("link", {name: "Gemachtigde toevoegen"}).click();
    await this.page.getByRole("textbox", { name: "Voorletters *" }).fill(authorisedAgent.initials);
    await this.page.getByRole("textbox", { name: "Voorvoegsel" }).fill(authorisedAgent.lastNamePrefix ?? "");
    await this.page.getByRole("textbox", { name: "Achternaam *" }).fill(authorisedAgent.lastName);
    await this.page.getByRole("button", {name: "Opslaan"}).click();
  }
}

  async editAuthorisedAgent(authorisedAgents: AuthorisedAgent[]) {
      for (const authorisedAgent of authorisedAgents) {
    await this.page.getByRole("cell", {name: authorisedAgent.lastName}).click();
    await this.page.getByRole("textbox", { name: "Voorletters *" }).fill(authorisedAgent.initials);
    await this.page.getByRole("textbox", { name: "Voorvoegsel" }).fill(authorisedAgent.lastNamePrefix ?? "");
    await this.page.getByRole("textbox", { name: "Achternaam *" }).fill(authorisedAgent.lastName);
    await this.page.getByRole("button", {name: "Opslaan"}).click();
  }
}

    async removeAuthorisedAgent(authorisedAgents: AuthorisedAgent[]) {
      for (const authorisedAgent of authorisedAgents) {
    await this.page.getByRole("cell", {name: authorisedAgent.lastName}).click();
    await this.page.getByRole("button", {name: "Gemachtigde verwijderen"}).click();
    await this.page.getByRole("button", {name: "Verwijderen", exact:true}).click();
  }
}
}