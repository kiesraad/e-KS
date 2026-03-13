import type { Locator, Page } from "@playwright/test";
import type { ListSubmitter } from "../models/listSubmitter";

export class SubstituteSubmittersPage {
  readonly buttonDelete: Locator;
  readonly buttonConfirmDelete: Locator;
  readonly buttonAdd: Locator;
  readonly buttonSave: Locator;
  readonly textfieldInitials: Locator;
  readonly textfieldLastNamePrefix: Locator;
  readonly textfieldLastName: Locator;

  constructor(protected readonly page: Page) {
    this.buttonDelete = this.page.getByRole("button", {
      name: "Vervanger voor het herstel van verzuimen verwijderen",
      exact: true,
    });
    this.buttonConfirmDelete = this.page.getByRole("button", {
      name: "Verwijderen",
      exact: true,
    });
    this.buttonAdd = this.page.getByRole("link", {
      name: "Vervanger voor het herstel van verzuimen",
    });
    this.buttonSave = this.page.getByRole("button", { name: "Opslaan" });
    this.textfieldInitials = this.page.getByRole("textbox", {
      name: "Voorletters",
    });
    this.textfieldLastNamePrefix = this.page.getByRole("textbox", {
      name: "Voorvoegsel",
    });
    this.textfieldLastName = this.page.getByRole("textbox", {
      name: "Achternaam",
    });
  }

  getSubmitterLocator(lastName: string) {
    return this.page.getByRole("link", { name: new RegExp(lastName) });
  }

  async deleteExistingSubstituteSubmitters() {
    //takes all links from table and saves href attributes of each link in list
    const hrefs = await this.page
      .locator(".substitute-list-submitters .person-block")
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

  async addSubstituteSubmitter(listSubmitter: ListSubmitter) {
    await this.buttonAdd.click();
    await this.textfieldInitials.fill(listSubmitter.initials);
    await this.textfieldLastNamePrefix.fill(listSubmitter.lastNamePrefix ?? "");
    await this.textfieldLastName.fill(listSubmitter.lastName);
    await this.buttonSave.click();
  }

  async editListSubmitter() {
    await this.buttonAdd.click();
    await this.textfieldInitials.fill("A");
    await this.textfieldLastNamePrefix.fill("de");
    await this.textfieldLastName.fill("Tester");
    await this.buttonSave.click();
  }
}
