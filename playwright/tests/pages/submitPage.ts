import type { Locator, Page } from "@playwright/test";

export class SubmitPage {
  readonly linkDownloadNl: Locator;
  readonly linkDownloadFry: Locator;
  readonly linkRegisteredDesignation: Locator;
  readonly linkNoLegalName: Locator;
  readonly linkBSN: Locator;
  readonly linkTooManyCandidates: Locator;
  readonly linkIncorrectDate: Locator;

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
    this.linkNoLegalName = this.page.getByRole("link", {
      name: "Te weinig gemachtigde/statutaire namen",
    });
    this.linkBSN = this.page.getByRole("link", {
      name: "BSN ontbreekt",
    });
    this.linkTooManyCandidates = this.page.getByRole("link", {
      name: "Te veel kandidaten",
    });
    this.linkIncorrectDate = this.page.getByRole("link", {
      name: "Geboortedatum lijkt onjuist",
    });
  }
}
