import { type Locator, type Page } from "@playwright/test";

export class CsbImportPage {
  readonly buttonImport: Locator;
  readonly buttonAddNew: Locator;
  readonly headerImport: Locator;
  readonly textfieldHashcode: Locator;


  constructor( readonly page: Page) {
    this.buttonImport = this.page.getByRole("button", {
      name: "Importeren",
    });
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
