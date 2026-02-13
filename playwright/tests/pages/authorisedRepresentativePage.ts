import { expect, type Page } from "@playwright/test";
import type { Candidate } from "../models/candidate";
import { AuthorisedRepresentative } from "../models/authorisedRepresentative";

export class AuthorisedRepresentativePage {
  private readonly page: Page;

  constructor(page: Page) {
    this.page = page;
  }

  async setAuthorisedRepresentative(authorisedRepresentative: AuthorisedRepresentative) {
    await this.page.getByLabel("Voorletters").fill(authorisedRepresentative.initials);
    await this.page
        .getByLabel("Voorvoegsel")
        .fill(authorisedRepresentative.lastNamePrefix ?? "");
    await this.page.getByLabel("Achternaam").fill(authorisedRepresentative.lastName);
        await this.page
        .getByRole("textbox", { name: "Postcode" })
        .fill(authorisedRepresentative.postalCode ?? "");
      await this.page
        .getByRole("textbox", { name: "Huisnummer", exact: true })
        .pressSequentially(authorisedRepresentative.houseNumber ?? "");
      await this.page
        .getByRole("textbox", { name: "Huisnummer toevoeging", exact: true })
        .press("Tab");
      await expect(
        this.page.getByRole("textbox", { name: "Straatnaam" }),
      ).toHaveValue(authorisedRepresentative.streetName ?? "");
      await expect(
        this.page.getByRole("combobox", { name: "Woonplaats" }),
      ).toHaveValue(authorisedRepresentative.locality ?? "");
    await this.page
        .getByRole("button", { name: "Opslaan en sluiten" })
        .click();
  }
}