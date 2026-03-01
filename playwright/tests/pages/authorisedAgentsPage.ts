import type { Page } from "@playwright/test";
import type { AuthorisedAgent } from "../models/authorisedAgent";

export class AuthorisedAgentsPage {
  private readonly page: Page;

  constructor(page: Page) {
    this.page = page;
  }

  getAgentLocator(lastName: string) {
    return this.page.getByRole("link", { name: new RegExp(lastName) });
  }

  async open() {
    await this.page.goto("/political-group/authorised-agents");
  }

  async deleteExistingAuthorisedAgents() {
    //takes all links from table and saves href attributes of each link in list
    const hrefs = await this.page
      .locator(".person-block")
      .evaluateAll((links) => links.map((link) => link.getAttribute("href")));

    for (const href of hrefs) {
      if (href) {
        await this.page.goto(href);
        await this.page
          .getByRole("button", { name: "Gemachtigde verwijderen", exact: true })
          .click();
        await this.page
          .getByRole("button", { name: "Verwijderen", exact: true })
          .click();
      }
    }
  }

  async addAuthorisedAgent(authorisedAgent: AuthorisedAgent) {
    await this.page
      .getByRole("link", { name: "Gemachtigde toevoegen" })
      .click();
    await this.page
      .getByRole("textbox", { name: "Voorletters" })
      .fill(authorisedAgent.initials);
    await this.page
      .getByRole("textbox", { name: "Voorvoegsel" })
      .fill(authorisedAgent.lastNamePrefix ?? "");
    await this.page
      .getByRole("textbox", { name: "Achternaam" })
      .fill(authorisedAgent.lastName);
    await this.page.locator("body").click();

    await this.page.getByRole("button", { name: "Opslaan" }).click();
  }

  async editAuthorisedAgent(authorisedAgents: AuthorisedAgent[]) {
    for (const authorisedAgent of authorisedAgents) {
      await this.page
        .getByRole("cell", { name: authorisedAgent.lastName })
        .click();
      await this.page
        .getByRole("textbox", { name: "Voorletters" })
        .fill(authorisedAgent.initials);
      await this.page
        .getByRole("textbox", { name: "Voorvoegsel" })
        .fill(authorisedAgent.lastNamePrefix ?? "");
      await this.page
        .getByRole("textbox", { name: "Achternaam" })
        .fill(authorisedAgent.lastName);
      await this.page.locator("body").click();

      await this.page.getByRole("button", { name: "Opslaan" }).click();
    }
  }
}
