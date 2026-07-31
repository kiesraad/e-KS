import { expect, type Locator, type Page } from "@playwright/test";


export class csbPoliticalGroupPage {
    readonly switchFinalize: Locator;
    readonly buttonRectifications: Locator;
    readonly buttonBack: Locator;
    readonly linkAllErrors: Locator;
    readonly buttonPaperCorrections: Locator;
    readonly linkGeneralInformation: Locator;
    readonly linkSupportDeclarations: Locator;
  

  constructor(protected readonly page: Page) {
    this.switchFinalize = this.page.getByLabel("Onderzoek afronden of heropenen");
    this.buttonRectifications = this.page.getByRole("button", {
      name: "Alle herstelacties",
    });
    this.buttonBack = this.page.getByRole("button", {
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
  }

  async selectedGroup(politicalgroup: string) {
    await expect(this.page.getByRole("heading", { name: politicalgroup })).toBeVisible();
  }
}


