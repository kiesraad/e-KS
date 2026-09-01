import type { Locator, Page } from "@playwright/test";

export class CsbCandidateListPage {
  readonly headerCandidateList: Locator;
  readonly linkAddOmission: Locator;
  readonly linkManageOmissions: Locator;

  constructor(protected readonly page: Page) {
    this.headerCandidateList = this.page.getByRole("heading", {
      name: "Kandidatenlijst",
    });
    this.linkAddOmission = this.page.getByRole("link", {
      name: "Verzuim toevoegen",
    });
    this.linkManageOmissions = this.page.getByRole("link", {
      name: "Verzuimen beheren",
    });
  }

  async getElectoralDistrict(page: Page, districts: string[]): Promise<string> {
    for (const district of districts) {
      if (await page.getByText(district).isVisible()) {
        return district;
      }
    }
    throw new Error(
      `None of the districts [${districts.join(", ")}] were visible on the page`,
    );
  }
}
