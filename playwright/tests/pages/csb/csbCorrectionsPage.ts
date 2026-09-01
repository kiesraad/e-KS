import type { Locator, Page } from "@playwright/test";

export class CsbCorrectionsPage {
  readonly textfieldCorrection: Locator;
  readonly buttonSaveCorrection: Locator;

  constructor(protected readonly page: Page) {
    this.textfieldCorrection = this.page.getByRole("textbox", {
      name: "Gecorrigeerde waarde",
    });
    this.buttonSaveCorrection = this.page.getByRole("button", {
      name: "Correctie opslaan",
    });
  }

  async addCorrection(correctedName: string) {
    await this.textfieldCorrection.fill(correctedName);
    await this.buttonSaveCorrection.click();
  }
}
