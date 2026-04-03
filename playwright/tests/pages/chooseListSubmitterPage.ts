import type { Locator, Page } from "@playwright/test";
import type { ListSubmitter } from "../models/listSubmitter";

export class ChooseListSubmitterPage {
  readonly linkElectoralDistrictsPage: Locator;
  readonly buttonClose: Locator;
  readonly buttonSave: Locator;
  readonly headerListSubmitter: Locator;

  constructor(protected readonly page: Page) {
    this.linkElectoralDistrictsPage = this.page.getByRole("link", {
      name: "Kieskringen",
    });
    this.buttonClose = this.page.getByRole("link", { name: "Sluiten" });
    this.buttonSave = this.page.getByRole("button", { name: "Opslaan" });
    this.headerListSubmitter = this.page.getByRole("heading", {
      name: "Gegevens lijstinleveraar",
    });
  }

  getSubmitterLocator(listSubmitter: ListSubmitter) {
    return this.page.getByRole("radio", {
      name: new RegExp(listSubmitter.lastName),
    });
  }

  getSubstituteSubmitterLocator(listSubmitter: ListSubmitter) {
    return this.page.getByRole("checkbox", {
      name: new RegExp(listSubmitter.lastName),
    });
  }

  async selectListSubmitter(listsubmitter: ListSubmitter) {
    await this.getSubmitterLocator(listsubmitter).check();
  }

  async selectSubstituteSubmitter(listsubmitter: ListSubmitter) {
    await this.getSubstituteSubmitterLocator(listsubmitter).check();
  }

  async removeSubstituteSubmitter(listsubmitter: ListSubmitter) {
    await this.getSubstituteSubmitterLocator(listsubmitter).uncheck();
  }
}
