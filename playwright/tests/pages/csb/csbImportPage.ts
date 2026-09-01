import type { Locator, Page } from "@playwright/test";

export class CsbImportPage {
  readonly buttonImport: Locator;
  readonly buttonImportAnyway: Locator;
  readonly buttonAddNew: Locator;
  readonly headerImport: Locator;
  readonly textfieldHashcode: Locator;
  readonly warningAlreadyImported: Locator;

  constructor(readonly page: Page) {
    this.buttonImport = this.page.getByRole("button", {
      name: "Importeren",
      exact: true,
    });
    this.buttonImportAnyway = this.page.getByRole("button", {
      name: "Toch importeren",
    });
    this.warningAlreadyImported = this.page.getByText("is al geïmporteerd");
    this.buttonAddNew = this.page.getByRole("button", {
      name: "Leeg aanmaken",
    });
    this.headerImport = this.page.getByRole("heading", {
      name: "Politieke groepering importeren",
    });
    this.textfieldHashcode = this.page.getByRole("textbox", {
      name: "Voer het begin van de hash code in",
    });
  }
}
