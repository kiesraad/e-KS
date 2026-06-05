import type { Locator, Page } from "@playwright/test";

export class ListDesignationPage {
  readonly selectStandalone: Locator;
  readonly selectCombined: Locator;
  readonly selectBlank: Locator;
  readonly buttonSaveAndNext: Locator;

  constructor(protected readonly page: Page) {
    this.selectStandalone = this.page.getByRole("radio", {
      name: "Op zichzelf staande geregistreerde naam",
    });
    this.selectCombined = this.page.getByRole("radio", {
      name: "Combinatie van meerdere geregistreerde namen",
    });
    this.selectBlank = this.page.getByRole("radio", {
      name: "Blanco lijst",
    });
    this.buttonSaveAndNext = this.page.getByRole("button", {
      name: "Opslaan en volgende",
    });
  }
}
