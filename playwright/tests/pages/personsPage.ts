import { expect, type Page } from "@playwright/test";
import type { Candidate } from "../models/candidate";
import { CreatePersonPage } from "./createPersonPage";
import { CorrespondenceAddressPage } from "./correspondenceAddressPage";
import { AuthorisedRepresentativePage } from "./authorisedRepresentativePage";

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
      if (candidate.authorisedRepresentative) {
        await new AuthorisedRepresentativePage(this.page).setAuthorisedRepresentative(candidate.authorisedRepresentative);
      }
      else {
        await new CorrespondenceAddressPage(this.page).setCorrespondenceAddress(candidate);
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
