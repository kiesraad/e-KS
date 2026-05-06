import type { Locator, Page } from "@playwright/test";

export class SubmitPage {
  readonly linkDownloadNl: Locator;
  readonly linkDownloadFry: Locator;

  constructor(protected readonly page: Page) {
    this.linkDownloadNl = this.page.getByRole("link", {
      name: "Alles in één zip",
    });
    this.linkDownloadFry = this.page.getByRole("link", {
      name: "Alles yn ien zip",
    });
  }
}
