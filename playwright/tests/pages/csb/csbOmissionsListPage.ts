import { expect, type Locator, type Page } from "@playwright/test";

export class csbOmissionsListPage {
  readonly linkAdd: Locator;
  readonly linkOverview: Locator;
  readonly buttonAuthoriseAppelation: Locator;
  readonly buttonAuthoriseCombination: Locator;
  readonly buttonAuthorisedAgent: Locator;
  readonly buttonAuthorisedAgentCombination: Locator;
  readonly textfieldTitle: Locator;
  readonly textfieldDescription: Locator;
  readonly textfieldLetter: Locator;
  readonly checkboxRecoverable: Locator;
  readonly buttonAddAndClose: Locator;
  readonly linkClose: Locator;
  readonly buttonRemoveOmission: Locator;
  readonly checkboxAllLists: Locator;

  constructor(protected readonly page: Page) {
    this.linkAdd = this.page.getByRole("link", { name: "Verzuimen toevoegen" });
    this.linkOverview = this.page.getByRole("link", { name: "Overzicht" });
    this.buttonAuthoriseAppelation = this.page.getByRole("button", {
      name: "De machtiging aanduiding ontbreekt",
    });
    this.buttonAuthoriseCombination = this.page.getByRole("button", {
      name: "De machtiging samenvoeging ontbreekt",
    });
    this.buttonAuthorisedAgent = this.page.getByRole("button", {
      name: "De gemachtigde is niet geregistreerd",
    });
    this.buttonAuthorisedAgentCombination = this.page.getByRole("button", {
      name: "De gemachtigde(n) is/zijn niet geregistreerd",
    });
    this.textfieldTitle = this.page.getByRole("textbox", {
      name: "Titel Verzuim",
    });
    this.textfieldDescription = this.page.getByRole("textbox", {
      name: "I1 verzuim toevoegen",
    });
    this.textfieldLetter = this.page.getByRole("textbox", {
      name: "Verzuimbriefnotitie toevoegen",
    });
    this.checkboxRecoverable = this.page.getByRole("checkbox", {
      name: "Herstelbaar verzuim",
    });
    this.buttonAddAndClose = this.page.getByRole("button", {
      name: "Toevoegen en sluiten",
    });
    this.linkClose = this.page.getByRole("link", { name: "Sluiten" });
    this.buttonRemoveOmission = this.page.getByRole("button", {
      name: "Verwijderen",
    });
    this.checkboxAllLists = this.page.getByRole("checkbox", {
      name: "Selecteer alle kandidatenlijsten",
    });
  }

  // verifies that the electoral district for the selected list is checked and districts for other lists are not checked
  async expectOnlySelectedDistrictChecked(
    page: Page,
    districts: string[],
    checkedDistrict: string,
  ) {
    for (const district of districts) {
      const checkbox = page.getByRole("checkbox", { name: district });
      if (district === checkedDistrict) {
        await expect(checkbox).toBeChecked();
      } else {
        await expect(checkbox).not.toBeChecked();
      }
    }
  }

  // verifies that the omission is only added for the selected electoral district
  async expectOnlySelectedDistrictAdded(
    page: Page,
    districts: string[],
    checkedDistrict: string,
  ) {
    for (const district of districts) {
      const text = page.getByText(district);
      if (district === checkedDistrict) {
        await expect(text).toBeVisible();
      } else {
        await expect(text).not.toBeVisible();
      }
    }
  }

  // verifies that the omission is added for all electoral districts
  async expectAllDistrictsAdded(page: Page, districts: string[]) {
    for (const district of districts) {
      const text = page.getByText(district);
      await expect(text).toBeVisible();
    }
  }

  // dispatchEvent fixes flakiness for webkit/firefox
  async clickRemoveOmission() {
    await this.buttonRemoveOmission.dispatchEvent("click");
  }
}
