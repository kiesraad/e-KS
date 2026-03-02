import type { Locator, Page } from "@playwright/test";

export class CandidateListsOverviewPage {
  readonly buttonAddList: Locator;
  readonly linkCandidateList: Locator;
  readonly headingAllCandidates: Locator;
  //readonly listDistricts: Locator;

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
    //this.listDistricts = this.page.getByRole("listitem", { name: district });
  }

  // async manageList() {
  //   await this.page
  //     .getByRole("link", { name: /^Kandidatenlijst \d+ \/ \d+/ })
  //     .first()
  //     .click();
  // }

  // async managePersons() {
  //   await this.page.getByRole("heading", { name: "Alle kandidaten" }).click();
  // }

  async getDistrictLocator(districtName: string) {
    return this.page.getByRole("listitem", { name: districtName });
  }
  // async checkRemovedDistricts(districts: string[]) {
  //   for (const district of districts) {
  //     await expect(
  //       this.page.getByRole("listitem", { name: district }),
  //     ).toHaveCount(0);
  //   }
  // }
}
