import type { Page } from "@playwright/test";
import type { Candidate } from "../models/candidate";

export class CreatePersonPage {
  private readonly page: Page;

  constructor(page: Page) {
    this.page = page;
  }

  async setPersonalDetails(candidate: Candidate) {
    await this.page.getByLabel("Voorletters").fill(candidate.initials);
    await this.page
      .getByLabel("Voorvoegsel")
      .fill(candidate.lastNamePrefix ?? "");
    await this.page.getByLabel("Achternaam").fill(candidate.lastName);
    await this.page.getByLabel("Roepnaam").fill(candidate.firstName ?? "");
    if (candidate.countryCode) {
      await this.page
        .getByRole("textbox", { name: "Landcode" })
        .fill(candidate.countryCode);
    }
    await this.page.getByLabel("Geslacht").selectOption(candidate.gender ?? "");
    await this.page
      .getByRole("textbox", { name: "Geboortedatum" })
      .fill(candidate.dateOfBirth ?? "");
    await this.page.locator("body").click();

    await this.page.getByRole("button", { name: "Volgende" }).click();
  }
}
