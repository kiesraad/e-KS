import type { Locator, Page } from "@playwright/test";

export class CsbGeneralInformationPage {
  readonly headerGeneralInformation: Locator;
  readonly linkAddOmission: Locator;
  readonly linkManageOmissions: Locator;
  readonly linkRegisteredDesignationStandalone: Locator;
  readonly linkRegisteredDesignationCombined: Locator;
  readonly textCorrectedNameStandalone: Locator;
  readonly textCorrectedNameCombined: Locator;
  readonly textCorrectedType: Locator;
  readonly linkBack: Locator;

  constructor(readonly page: Page) {
    this.headerGeneralInformation = this.page.getByRole("heading", {
      name: "Basisgegevens",
      exact: true,
    });
    this.linkAddOmission = this.page.getByRole("link", {
      name: "Verzuim toevoegen",
    });
    this.linkManageOmissions = this.page.getByRole("link", {
      name: "Overzicht",
    });
    this.linkRegisteredDesignationStandalone = this.page.getByRole("cell", {
      name: "Geregistreerde aanduiding",
    });
    this.linkRegisteredDesignationCombined = this.page.getByRole("cell", {
      name: "Samengevoegde aanduiding",
    });
    this.textCorrectedNameStandalone = this.page
      .getByRole("row", { name: "Geregistreerde aanduiding:" })
      .getByRole("strong");
    this.textCorrectedNameCombined = this.page
      .getByRole("row", { name: "Samengevoegde aanduiding:" })
      .getByRole("strong");
    this.textCorrectedType = this.page
      .getByRole("row", { name: "Type lijstaanduiding:" })
      .getByRole("strong");
    this.linkBack = this.page.getByRole("link", { name: "Terug" });
  }
}
