import { expect, type Page } from "@playwright/test";
import type { Candidate } from "../models/candidate";

export class ManageCandidateListPage {
  private readonly page: Page;

  constructor(page: Page) {
    this.page = page;
  }

  async addExistingCandidates(candidates: string[]) {
    for (const candidate of candidates) {
      await this.page.getByRole("link", { name: "Existing" }).click();
      await this.page
        .getByRole("row", { name: candidate })
        .getByRole("button")
        .click();
      await expect(
        this.page.getByRole("cell", { name: candidate }),
      ).toBeVisible();
    }
  }

  async addNewCandidates(candidates: Candidate[]) {
    for (const candidate of candidates) {
      await this.page.getByRole("link", { name: "New" }).click();
      await this.page.getByLabel("Initials").fill(candidate.initials);
      await this.page
        .locator('input[name="last_name"]')
        .fill(candidate.lastName);
      await this.page.getByLabel("First name").fill(candidate.firstName ?? "");
      await this.page.getByLabel("Country code").fill("NL");
      await this.page.getByRole("button", { name: "Save" }).click();
      await this.page.getByLabel("Locality").fill(candidate.locality ?? "");
      await this.page.getByRole("button", { name: "Save" }).click();
      await expect(
        this.page.getByRole("cell", { name: candidate.lastName }),
      ).toBeVisible();
    }
  }

  async removeDistricts(districts: string[]) {
    await this.page.getByRole("link", { name: "List details" }).click();
    for (const district of districts) {
      await this.page.getByRole("checkbox", { name: district }).uncheck();
    }
    await this.page.getByRole("button", { name: "Save" }).click();
  }

  async addDistricts(districts: string[]) {
    await this.page.getByRole("link", { name: "List details" }).click();
    for (const district of districts) {
      await this.page.getByRole("checkbox", { name: district }).check();
    }
    await this.page.getByRole("button", { name: "Save" }).click();
  }

  async removeList(districts: string[]) {
    await this.page.getByRole("link", { name: "List details" }).click();
    await this.page
      .getByRole("button", { name: "Delete candidate list" })
      .click();
    await this.page
      .getByRole("button", { name: "Delete", exact: true })
      .click();
    for (const district of districts) {
      await expect(
        this.page.getByRole("listitem", { name: district }),
      ).toHaveCount(0);
    }
  }
}
