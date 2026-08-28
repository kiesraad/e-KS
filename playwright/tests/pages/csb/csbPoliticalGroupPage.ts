import { expect, type Locator, type Page } from "@playwright/test";

export class CsbPoliticalGroupPage {
  readonly switchFinalize: Locator;
  readonly buttonRectifications: Locator;
  readonly buttonBack: Locator;
  readonly linkAllErrors: Locator;
  readonly buttonPaperCorrections: Locator;
  readonly linkGeneralInformation: Locator;
  readonly linkSupportDeclarations: Locator;
  readonly linkCandidateList: Locator;
  readonly linkDelete: Locator;
  readonly buttonDeleteConfirm: Locator;

  constructor(protected readonly page: Page) {
    this.switchFinalize = this.page.getByLabel(
      "Onderzoek afronden of heropenen",
    );
    this.buttonRectifications = this.page.getByRole("button", {
      name: "Alle herstelacties",
    });
    this.buttonBack = this.page.getByRole("link", {
      name: "Terug",
    });
    this.linkAllErrors = this.page.getByRole("link", {
      name: "Alle BRP fouten controleren",
    });
    this.buttonPaperCorrections = this.page.getByRole("button", {
      name: "Correcties overnemen",
    });
    this.linkGeneralInformation = this.page.getByRole("link", {
      name: "Basisgegevens controleren",
    });
    this.linkSupportDeclarations = this.page.getByRole("link", {
      name: "Verzuimen beheren",
    });
    this.linkCandidateList = this.page.getByRole("link", {
      name: "Kandidatenlijst controleren",
    });
    this.linkDelete = this.page.getByRole("link", { name: "Verwijderen" });
    this.buttonDeleteConfirm = this.page.getByRole("button", {
      name: "Verwijderen",
    });
  }

  async selectedGroup(politicalgroup: string) {
    await expect(
      this.page.getByRole("heading", { name: politicalgroup }),
    ).toBeVisible();
  }

  async deleteGroup() {
    await this.linkDelete.click();
    await this.buttonDeleteConfirm.click();
    // wait for the redirect to the overview, so the test does not end while
    // the delete request is still in flight
    await this.page.waitForURL(/\/csb\/examination$/);
  }
}
