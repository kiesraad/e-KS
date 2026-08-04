import type { Locator, Page } from "@playwright/test";

export class CsbGeneralInformationPage {
  readonly headerGeneralInformation: Locator;
  readonly linkAddOmission: Locator;
  readonly linkManageOmissions: Locator;
  readonly linkRegisteredDesignation: Locator;
  readonly textCorrectedName: Locator;

  constructor(readonly page: Page) {
    this.headerGeneralInformation = this.page.getByRole("heading", {
      name: "Basisgegevens",
    });
    this.linkAddOmission = this.page.getByRole("link", {
      name: "Verzuim toevoegen",
    });
    this.linkManageOmissions = this.page.getByRole("link", {
      name: "Verzuimen beheren",
    });
    this.linkRegisteredDesignation = this.page.getByRole("cell", {
      name: "Geregistreerde aanduiding",
    });
    this.textCorrectedName = this.page
      .getByRole("row", { name: "Geregistreerde aanduiding:" })
      .getByRole("strong");
  }
}
