import type { Locator, Page } from "@playwright/test";
import type { AuthorisedAgent } from "../models/authorisedAgent";

export class AuthorisedAgentsPage {
  readonly textfieldInitials: Locator;
  readonly textfieldLastNamePrefix: Locator;
  readonly textfieldLastName: Locator;
  readonly buttonDelete: Locator;
  readonly buttonConfirmDelete: Locator;
  readonly buttonAdd: Locator;
  readonly buttonSave: Locator;

  constructor(protected readonly page: Page) {
    this.textfieldInitials = this.page.getByLabel("Voorletters");
    this.textfieldLastNamePrefix = this.page.getByLabel("Voorvoegsel");
    this.textfieldLastName = this.page.getByLabel("Achternaam");
    this.buttonDelete = this.page.getByRole("button", {
      name: "Gemachtigde verwijderen",
      exact: true,
    });
    this.buttonConfirmDelete = this.page.getByRole("button", {
      name: "Verwijderen",
      exact: true,
    });
    this.buttonAdd = this.page.getByRole("link", {
      name: "Gemachtigde toevoegen",
    });
    this.buttonSave = this.page.getByRole("button", { name: "Opslaan" });
  }

  getAgentLocator(lastName: string) {
    return this.page.getByRole("link", { name: new RegExp(lastName) });
  }

  async deleteExistingAuthorisedAgents() {
    //takes all links from table and saves href attributes of each link in list
    const hrefs = await this.page
      .locator(".person-block")
      .evaluateAll((links) => links.map((link) => link.getAttribute("href")));

    for (const href of hrefs) {
      if (href) {
        await this.page.goto(href);
        await this.buttonDelete.click();
        await this.buttonConfirmDelete.click();
      }
    }
  }

  async addAuthorisedAgent(authorisedAgent: AuthorisedAgent) {
    await this.buttonAdd.click();
    await this.textfieldInitials.fill(authorisedAgent.initials);
    await this.textfieldLastNamePrefix.fill(
      authorisedAgent.lastNamePrefix ?? "",
    );
    await this.textfieldLastName.fill(authorisedAgent.lastName);
    await this.buttonSave.click();
  }

  async editAuthorisedAgent(authorisedAgents: AuthorisedAgent[]) {
    for (const authorisedAgent of authorisedAgents) {
      await this.textfieldInitials.fill(authorisedAgent.initials);
      await this.textfieldLastNamePrefix.fill(
        authorisedAgent.lastNamePrefix ?? "",
      );
      await this.textfieldLastName.fill(authorisedAgent.lastName);
      await this.buttonSave.click();
    }
  }
}
