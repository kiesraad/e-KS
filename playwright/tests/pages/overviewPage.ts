import { expect, type Locator, type Page } from "@playwright/test";

export class OverviewPage {
  readonly buttonSwitchElection: Locator;
  readonly linkGeneralInformation: Locator;
  readonly linkCandidateList: Locator;
  readonly linkFinalise: Locator;
  readonly linkAuditLog: Locator;
  readonly linkLogout: Locator;
  readonly buttonLanguageNL: Locator;
  readonly buttonLanguageEN: Locator;

  constructor(protected readonly page: Page) {
    this.buttonSwitchElection = this.page.getByRole("link", {
      name: "Verkiezing wisselen",
    });
    this.linkGeneralInformation = this.page.getByRole("link", {
      name: "Stap 1",
    });
    this.linkCandidateList = this.page.getByRole("link", {
      name: "Stap 2",
    });
    this.linkFinalise = this.page.getByRole("link", {
      name: "Stap 3",
    });
    this.linkAuditLog = this.page.getByRole("link", {
      name: "Logboek",
    });
    this.linkLogout = this.page.getByRole("link", {
      name: "Afmelden",
    });
    this.buttonLanguageNL = this.page.getByRole("button", {
      name: "NL",
    });
    this.buttonLanguageEN = this.page.getByRole("button", {
      name: "EN",
    });
  }

  async selectedElection(election: string) {
    await expect(
      this.page.getByRole("heading", { name: election }),
    ).toBeVisible();
  }
}
