import type { Locator, Page } from "@playwright/test";

export class EditListDetailsPage {
  readonly buttonNext: Locator;
  readonly buttonClose: Locator;
  readonly buttonAdd: Locator;
  readonly buttonRemoveList: Locator;
  readonly buttonConfirmRemoveList: Locator;

  constructor(protected readonly page: Page) {
    this.buttonNext = this.page.getByRole("button", { name: "Volgende" });
    this.buttonClose = this.page.getByRole("link", { name: "Sluiten" }).first();
    this.buttonAdd = this.page.getByRole("button", { name: "Toevoegen" });
     this.buttonRemoveList = this.page.getByRole("button", {
      name: "Kandidatenlijst verwijderen",
    });
    this.buttonConfirmRemoveList = this.page.getByRole("button", {
      name: "Verwijderen",
      exact: true,
    });
  }

  async addDistricts(districts: string[]) {
    for (const district of districts) {
      await this.page.getByRole("checkbox", { name: district }).check();
    }
    await this.buttonNext.click();
    await this.page.waitForURL("**/list-submitter**");
    await this.buttonAdd.click();
  }

  async removeDistricts(districts: string[]) {
    for (const district of districts) {
      await this.page.getByRole("checkbox", { name: district }).uncheck();
    }
    await this.buttonNext.click();
    await this.page.waitForURL("**/list-submitter**");
    await this.buttonAdd.click();
  }

  async removeList() {
    await this.buttonRemoveList.click();
    await this.buttonConfirmRemoveList.click();
  }
  
}
