import { expect, type Page } from "@playwright/test";
import type { Candidate } from "../models/candidate";

export class PersonsPage {
  private readonly page: Page;

  constructor(page: Page) {
    this.page = page;
  }

  async open() {
    await this.page.goto("/persons");
  }

  async addPersons(candidates: Candidate[]) {
    for (const candidate of candidates) {
      await this.page.getByRole("link", { name: "Persoon toevoegen" }).click();
      await this.page.getByLabel("Voorletters").fill(candidate.initials);
      await this.page
        .getByLabel("Voorvoegsel")
        .fill(candidate.lastNamePrefix ?? "");
      await this.page.getByLabel("Achternaam").fill(candidate.lastName);
      await this.page.getByLabel("Roepnaam").fill(candidate.firstName ?? "");
      await this.page
        .getByLabel("Geslacht")
        .selectOption(candidate.gender ?? "");
      await this.page
        .getByRole("textbox", { name: "Geboortedatum" })
        .fill(candidate.dateOfBirth ?? "");
      await this.page.getByLabel("Landcode").fill("NL");

      // enable the submit button (only active if all required fields are filled in)
      await this.page.getByLabel("Landcode").press("Tab");

      await this.page
        .getByRole("button", { name: "Opslaan en verder" })
        .click();
      await this.page
        .getByRole("textbox", { name: "Postcode" })
        .fill(candidate.postalCode ?? "");
      await this.page
        .getByRole("textbox", { name: "Huisnummer", exact: true })
        .fill(candidate.houseNumber ?? "");
      await this.page
        .getByRole("textbox", { name: "Huisnummer toevoeging", exact: true })
        .press("Tab");
      await expect(
        this.page.getByRole("textbox", { name: "Straatnaam" }),
      ).toHaveValue(candidate.streetName ?? "");
      await expect(
        this.page.getByRole("combobox", { name: "Woonplaats" }),
      ).toHaveValue(candidate.locality ?? "");
      await this.page.getByLabel("Woonplaats").fill(candidate.locality ?? "");
      await this.page
        .getByRole("button", { name: "Opslaan en sluiten" })
        .click();
    }
  }

  async checkPerson(candidates: Candidate[]) {
    for (const candidate of candidates) {
      await expect(
        this.page.getByRole("cell", { name: candidate.lastName }).first(),
      ).toBeVisible();
    }
  }
}
