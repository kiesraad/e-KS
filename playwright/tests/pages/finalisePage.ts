import type { Locator, Page } from "@playwright/test";

export class FinalisePage {
  readonly linkDownloadNl: Locator;
  readonly linkDownloadFry: Locator;
  readonly linkRegisteredDesignation: Locator;
  readonly linkNoLegalName: Locator;
  readonly linkBSN: Locator;
  readonly linkTooManyCandidates: Locator;
  readonly linkIncorrectDate: Locator;
  readonly linkCandidateList: Locator;

  constructor(protected readonly page: Page) {
    this.linkDownloadNl = this.page.getByRole("link", {
      name: "Alles in één zip",
    });
    this.linkDownloadFry = this.page.getByRole("link", {
      name: "Alles yn ien zip",
    });
    this.linkRegisteredDesignation = this.page.getByRole("link", {
      name: "Geregistreerde aanduiding",
    });
    this.linkNoLegalName = this.page.getByRole("link", {
      name: "1 statutaire naam te weinig",
    });
    this.linkBSN = this.page.getByRole("link", {
      name: "BSN",
    });
    this.linkTooManyCandidates = this.page.getByRole("link", {
      name: /\d+ (kandidaat|kandidaten) te veel/,
    });
    this.linkIncorrectDate = this.page.getByRole("link", {
      name: "Geboortedatum lijkt onjuist",
    });
    this.linkCandidateList = this.page.getByRole("link", {
      name: "Kandidatenlijst",
    });
  }
}
