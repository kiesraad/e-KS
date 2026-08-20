import type { Locator, Page } from "@playwright/test";

export class CsbExaminationPage {
  readonly buttonFinalise: Locator;
  readonly linkAddPoliticalGroup: Locator;
  readonly headerExamination: Locator;
  readonly buttonI1: Locator;
  readonly buttonI4: Locator;

  constructor(protected readonly page: Page) {
    this.buttonFinalise = this.page.getByRole("button", {
      name: "Afronden",
    });
    this.linkAddPoliticalGroup = this.page.getByRole("link", {
      name: "Politieke groepering toevoegen",
    });
    this.headerExamination = this.page.getByRole("heading", {
      name: "Onderzoek",
      exact: true,
    });
    this.buttonI1 = this.page.getByRole("button", { name: "Print I 1" });
    this.buttonI4 = this.page.getByRole("button", { name: "Print I 4" });
  }

  async selectPoliticalGroup(politicalgroup: string) {
    await this.page
      .getByRole("cell", { name: politicalgroup, exact: true })
      .click();
  }
}
