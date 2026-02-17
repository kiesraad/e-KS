import { expect, type Page } from "@playwright/test";
import type { Candidate } from "../models/candidate";
import { AuthorisedPersonPage } from "./authorisedPersonPage";
import { CorrespondenceAddressPage } from "./correspondenceAddressPage";
import { CreatePersonPage } from "./createPersonPage";

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
      await new CreatePersonPage(this.page).setPersonalDetails(candidate);
      if (candidate.authorisedPerson) {
        await new AuthorisedPersonPage(this.page).setAuthorisedPerson(
          candidate.authorisedPerson,
        );
      } else {
        await new CorrespondenceAddressPage(this.page).setCorrespondenceAddress(
          candidate,
        );
      }
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
