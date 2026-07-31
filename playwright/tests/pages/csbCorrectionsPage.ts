import { expect, type Locator, type Page } from "@playwright/test";


export class csbCorrectionsPage {
    readonly textfieldCorrection: Locator;
    readonly buttonSaveCorrection: Locator;
  

  constructor(protected readonly page: Page) {
    this.textfieldCorrection = this.page.getByRole("textbox", { name: "Gecorrigeerde waarde" });
    this.buttonSaveCorrection = this.page.getByRole("button", {
      name: "Correctie opslaan",
    });
    
  }

  async addCorrection(correctedName: string) {
    await this.textfieldCorrection.fill(correctedName);
    await this.buttonSaveCorrection.click();
  }
}


