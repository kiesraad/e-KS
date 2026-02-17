import { expect, type Page } from "@playwright/test";
import type { AuthorisedPerson } from "../models/authorisedPerson";

export class AuthorisedPersonPage {
  private readonly page: Page;

  constructor(page: Page) {
    this.page = page;
  }

  async setAuthorisedPerson(authorisedPerson: AuthorisedPerson) {
    await this.page.getByLabel("Voorletters").fill(authorisedPerson.initials);
    await this.page
      .getByLabel("Voorvoegsel")
      .fill(authorisedPerson.lastNamePrefix ?? "");
    await this.page.getByLabel("Achternaam").fill(authorisedPerson.lastName);
    await this.page
      .getByRole("textbox", { name: "Postcode" })
      .fill(authorisedPerson.postalCode ?? "");
    await this.page
      .getByRole("textbox", { name: "Huisnummer", exact: true })
      .pressSequentially(authorisedPerson.houseNumber ?? "");
    await this.page
      .getByRole("textbox", { name: "Huisnummer toevoeging", exact: true })
      .press("Tab");
    await expect(
      this.page.getByRole("textbox", { name: "Straatnaam" }),
    ).toHaveValue(authorisedPerson.streetName ?? "");
    await expect(
      this.page.getByRole("combobox", { name: "Woonplaats" }),
    ).toHaveValue(authorisedPerson.locality ?? "");
    await this.page.getByRole("button", { name: "Toevoegen" }).click();
  }
}
