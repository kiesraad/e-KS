import type { Locator, Page } from "@playwright/test";

export class SubmitPage {
  readonly linkDownloadNl: Locator;
  readonly linkDownloadFry: Locator;
  readonly linkRegisteredDesignation: Locator;
  readonly linkAuthorisedAgent: Locator;
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
    this.linkAuthorisedAgent = this.page.getByRole("link", {
      name: "Geen gemachtigde toegevoegd",
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
