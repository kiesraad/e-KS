import { expect, type Page } from "@playwright/test";
import type { Candidate } from "../models/candidate";

export class CorrespondenceAddressPage {
  private readonly page: Page;

  constructor(page: Page) {
    this.page = page;
  }

  async setCorrespondenceAddress(candidate: Candidate) {
    await this.page
      .getByRole("textbox", { name: "Postcode" })
      .fill(candidate.postalCode ?? "");
    await this.page
      .getByRole("textbox", { name: "Huisnummer", exact: true })
      .pressSequentially(candidate.houseNumber ?? "");
    await this.page
      .getByRole("textbox", {
        name: "Huisnummer toevoeging",
        exact: true,
      })
      .pressSequentially(candidate.houseNumberAddition ?? "");
    await this.page.locator("body").click();
    await expect(
      this.page.getByRole("textbox", { name: "Straatnaam" }),
    ).toHaveValue(candidate.streetName ?? "");
    await expect(
      this.page.getByRole("combobox", { name: "Woonplaats" }),
    ).toHaveValue(candidate.locality ?? "");

    await this.page.getByRole("button", { name: "Toevoegen" }).click();
  }
}
