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
    for (var candidate of candidates) {
      await this.page.getByRole("link", { name: "Add person" }).click();
      await this.page.getByLabel("Initials").fill(candidate.initials);
      await this.page
        .locator('input[name="last_name_prefix"]')
        .fill(candidate.lastNamePrefix ?? "");
      await this.page
        .locator('input[name="last_name"]')
        .fill(candidate.lastName);
      await this.page.getByLabel("First name").fill(candidate.firstName ?? "");
      await this.page.getByLabel("Gender").selectOption(candidate.gender ?? "");
      await this.page
        .getByRole("textbox", { name: "Date of birth" })
        .fill(candidate.dateOfBirth ?? "");
      await this.page.getByRole("button", { name: "Save" }).click();
      await this.page
        .getByRole("textbox", { name: "Postal code" })
        .fill(candidate.postalCode ?? "");
      await this.page
        .getByRole("textbox", { name: "House number", exact: true })
        .fill(candidate.houseNumber ?? "");
      await this.page
        .getByRole("textbox", { name: "House number", exact: true })
        .press("Tab");
      await expect(
        this.page.getByRole("textbox", { name: "Street name" }),
      ).toHaveValue(candidate.streetName ?? "");
      await expect(
        this.page.getByRole("combobox", { name: "Locality" }),
      ).toHaveValue(candidate.locality ?? "");
      await this.page.getByLabel("Locality").fill(candidate.locality ?? "");
      await this.page.getByRole("button", { name: "Save" }).click();
    }
  }

  async checkPerson(candidates: Candidate[]) {
    for (var candidate of candidates) {
      await expect(
        this.page.getByRole("cell", { name: candidate.lastName }).first(),
      ).toBeVisible();
    }
  }
}
