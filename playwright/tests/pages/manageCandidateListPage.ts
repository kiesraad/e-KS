import { expect, type Page } from "@playwright/test";
import type { Candidate } from "../models/candidate";

export class ManageCandidateListPage {
  private readonly page: Page;

  constructor(page: Page) {
    this.page = page;
  }

  async addExistingCandidates(candidates: string[]) {
    for (const candidate of candidates) {
      await this.page.getByRole("link", { name: "Bestaande" }).click();

      // search first part of the name
      await this.page
        .getByLabel("Zoek bestaande persoon")
        .pressSequentially(candidate.slice(0, 5));

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
      await this.page.getByRole("link", { name: "Nieuwe" }).click();
      await this.page.getByLabel("Voorletters").fill(candidate.initials);
      await this.page.getByLabel("Achternaam").fill(candidate.lastName);
      await this.page.getByLabel("Roepnaam").fill(candidate.firstName ?? "");

      await this.page.locator("body").click();
      await this.page.getByRole("button", { name: "Opslaan" }).click();
      await this.page
        .getByRole("link", { name: "Correspondentieadres" })
        .click();

      await this.page.getByLabel("Woonplaats").fill(candidate.locality ?? "");
      await this.page.locator("body").click();

      await this.page.getByRole("button", { name: "Opslaan" }).click();
      await this.page.getByRole("link", { name: "Sluiten" }).first().click();

      await expect(
        this.page.getByRole("cell", { name: candidate.lastName }),
      ).toBeVisible();
    }
  }

  async removeDistricts(districts: string[]) {
    await this.page
      .getByRole("main")
      .getByRole("link", { name: "Lijstgegevens" })
      .click();
    for (const district of districts) {
      await this.page.getByRole("checkbox", { name: district }).uncheck();
    }
    await this.page.getByRole("button", { name: "Opslaan" }).click();
  }

  async addDistricts(districts: string[]) {
    await this.page
      .getByRole("main")
      .getByRole("link", { name: "Lijstgegevens" })
      .click();
    for (const district of districts) {
      await this.page.getByRole("checkbox", { name: district }).check();
    }
    await this.page.getByRole("button", { name: "Opslaan" }).click();
  }

  async removeList(districts: string[]) {
    await this.page
      .getByRole("main")
      .getByRole("link", { name: "Lijstgegevens" })
      .click();
    await this.page
      .getByRole("button", { name: "Kandidatenlijst verwijderen" })
      .click();
    await this.page
      .getByRole("button", { name: "Verwijderen", exact: true })
      .click();
    for (const district of districts) {
      await expect(
        this.page.getByRole("listitem", { name: district }),
      ).toHaveCount(0);
    }
  }
}
