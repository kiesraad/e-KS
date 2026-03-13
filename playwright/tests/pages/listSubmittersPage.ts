import type { Locator, Page } from "@playwright/test";
import type { ListSubmitter } from "../models/listSubmitter";

export class ListSubmittersPage {
  readonly buttonDelete: Locator;
  readonly buttonConfirmDelete: Locator;
  readonly buttonAdd: Locator;
  readonly buttonSave: Locator;
  readonly textfieldInitials: Locator;
  readonly textfieldLastNamePrefix: Locator;
  readonly textfieldLastName: Locator;

  constructor(protected readonly page: Page) {
    this.buttonDelete = this.page.getByRole("button", {
      name: "Lijstinleveraar verwijderen",
      exact: true,
    });
    this.buttonConfirmDelete = this.page.getByRole("button", {
      name: "Verwijderen",
      exact: true,
    });
    this.buttonAdd = this.page.getByRole("link", {
      name: "Lijstinleveraar toevoegen",
    });
    this.buttonSave = this.page.getByRole("button", { name: "Opslaan" });
    this.textfieldInitials = this.page.getByLabel("Voorletters");
    this.textfieldLastNamePrefix = this.page.getByLabel("Voorvoegsel");
    this.textfieldLastName = this.page.getByLabel("Achternaam");
  }

  getSubmitterLocator(lastName: string) {
    return this.page.getByRole("link", { name: new RegExp(lastName) });
  }

  async deleteExistingListSubmitters() {
    //takes all links from table and saves href attributes of each link in list
    const hrefs = await this.page
      .locator(".list-submitters .person-block")
      .evaluateAll((links) => links.map((link) => link.getAttribute("href")));

    for (const href of hrefs) {
      if (href) {
        await this.page.goto(href);
        await this.buttonDelete.click();
        await this.buttonConfirmDelete.click();
        await this.page.waitForURL("**/list-submitters");
      }
    }
  }

  async addListSubmitter(listSubmitter: ListSubmitter) {
    await this.buttonAdd.click();
    await this.textfieldInitials.fill(listSubmitter.initials);
    await this.textfieldLastNamePrefix.fill(listSubmitter.lastNamePrefix ?? "");
    await this.textfieldLastName.fill(listSubmitter.lastName);
    await this.buttonSave.click();
  }
}
