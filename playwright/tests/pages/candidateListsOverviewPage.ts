import type { Locator, Page } from "@playwright/test";

export class CandidateListsOverviewPage {
  readonly buttonAddList: Locator;
  readonly linkCandidateList: Locator;
  readonly headingAllCandidates: Locator;

  constructor(protected readonly page: Page) {
    this.buttonAddList = this.page.getByRole("link", {
      name: "Lijst aanmaken",
    });
    this.linkCandidateList = this.page.getByRole("link", {
      name: /^Kandidatenlijst \d+ \/ \d+/,
    });
    this.headingAllCandidates = this.page.getByRole("heading", {
      name: "Alle kandidaten",
    });
  }

  async getDistrictLocator(districtName: string) {
    return this.page.getByRole("listitem", { name: districtName });
  }
}
