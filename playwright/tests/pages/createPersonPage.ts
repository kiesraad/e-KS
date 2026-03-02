import type { Locator, Page } from "@playwright/test";
import type { Candidate } from "../models/candidate";

export class CreatePersonPage {
  readonly textfieldInitials: Locator;
  readonly textfieldLastNamePrefix: Locator;
  readonly textfieldLastName: Locator;
  readonly textfieldFirstName: Locator;
  readonly textfieldCountryCode: Locator;
  readonly selectGender: Locator;
  readonly textfieldDateOfBirth: Locator;
  readonly buttonNext: Locator;

  constructor(protected readonly page: Page) {
    this.textfieldInitials = this.page.getByLabel("Voorletters");
    this.textfieldLastNamePrefix = this.page.getByLabel("Voorvoegsel");
    this.textfieldLastName = this.page.getByLabel("Achternaam");
    this.textfieldFirstName = this.page.getByLabel("Roepnaam");
    this.textfieldCountryCode = this.page.getByRole("textbox", {
      name: "Landcode",
    });
    this.selectGender = this.page.getByLabel("Geslacht");
    this.textfieldDateOfBirth = this.page.getByRole("textbox", {
      name: "Geboortedatum",
    });
    this.buttonNext = this.page.getByRole("button", { name: "Volgende" });
  }

  async setPersonalDetails(candidate: Candidate) {
    await this.textfieldInitials.fill(candidate.initials);
    await this.textfieldLastNamePrefix.fill(candidate.lastNamePrefix ?? "");
    await this.textfieldLastName.fill(candidate.lastName);
    await this.textfieldFirstName.fill(candidate.firstName ?? "");
    if (candidate.countryCode) {
      await this.textfieldCountryCode.fill(candidate.countryCode);
    }
    await this.selectGender.selectOption(candidate.gender ?? "");
    await this.textfieldDateOfBirth.fill(candidate.dateOfBirth ?? "");

    await this.buttonNext.click();
  }
}
