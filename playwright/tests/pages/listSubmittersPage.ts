import type { Locator, Page } from "@playwright/test";
import type { ListSubmitter } from "../models/listSubmitter";

export class ListSubmittersPage {
  readonly buttonAdd: Locator;
  readonly buttonSave: Locator;
  readonly textfieldInitials: Locator;
  readonly textfieldLastNamePrefix: Locator;
  readonly textfieldLastName: Locator;
  readonly linkEditSubmitter: Locator;
  readonly buttonNext: Locator;

  constructor(protected readonly page: Page) {
    this.buttonAdd = this.page.getByRole("link", {
      name: "Lijstinleveraar toevoegen",
    });
    this.buttonSave = this.page.getByRole("button", { name: "Opslaan" });
    this.textfieldInitials = this.page.getByLabel("Voorletters");
    this.textfieldLastNamePrefix = this.page.getByLabel("Voorvoegsel");
    this.textfieldLastName = this.page.getByLabel("Achternaam");
    // On the view page the single list submitter is rendered as a
    // .person-block link that opens the edit overlay.
    this.linkEditSubmitter = this.page.locator("a.person-block").first();
    this.buttonNext = this.page.getByRole("link", { name: "Verder naar het startscherm" });
  }

  getSubmitterLocator(lastName: string) {
    return this.page.getByRole("link", { name: new RegExp(lastName) });
  }

  async openEditor() {
    // Prefer the "add" button when the submitter is still empty,
    // otherwise click the existing submitter block to edit it.
    if (await this.buttonAdd.isVisible()) {
      await this.buttonAdd.click();
    } else {
      await this.linkEditSubmitter.click();
    }
  }

  async setListSubmitter(listSubmitter: ListSubmitter) {
    await this.openEditor();
    await this.textfieldInitials.fill(listSubmitter.initials);
    await this.textfieldLastNamePrefix.fill(listSubmitter.lastNamePrefix ?? "");
    await this.textfieldLastName.fill(listSubmitter.lastName);
    await this.buttonSave.click();
    await this.page.waitForURL("**/list-submitter**");
  }
}
