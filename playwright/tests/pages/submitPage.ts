import type { Locator, Page } from "@playwright/test";

export class SubmitPage {
  readonly linkDownloadSingle: Locator;
  readonly linkDownloadNl: Locator;
  readonly linkDownloadFry: Locator;

  constructor(protected readonly page: Page) {
    this.linkDownloadSingle = this.page.getByRole("link", {
      name: "Alles in één zip",
    });
    this.linkDownloadNl = this.page.getByRole("link", {
      name: "Alles in één zip (Nederlands)",
    });
    this.linkDownloadFry = this.page.getByRole("link", {
      name: "Alles in één zip (Fries)",
    });
  }
}
