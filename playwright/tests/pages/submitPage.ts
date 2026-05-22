import type { Locator, Page } from "@playwright/test";

export class SubmitPage {
  readonly linkDownloadNl: Locator;
  readonly linkDownloadFry: Locator;
  readonly linkRegisteredDesignation: Locator;
  readonly linkBSN: Locator;

  constructor(protected readonly page: Page) {
    this.linkDownloadNl = this.page.getByRole("link", {
      name: "Alles in één zip",
    });
    this.linkDownloadFry = this.page.getByRole("link", {
      name: "Alles yn ien zip",
    });
    this.linkRegisteredDesignation = this.page.getByRole("link", {
      name: "Geregistreerde aanduiding ontbreekt",
    });
    this.linkBSN = this.page.getByRole("link", {
      name: "BSN ontbreekt",
    });
  }
}
