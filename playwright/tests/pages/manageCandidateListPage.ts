import type { Locator, Page } from "@playwright/test";
import type { Candidate } from "../models/candidate";

export class ManageCandidateListPage {
  readonly buttonAddExistingCandidate: Locator;
  readonly buttonAddNewCandidate: Locator;
  readonly buttonSearchExistingCandidate: Locator;
  readonly textfieldInitials: Locator;
  readonly textfieldLastName: Locator;
  readonly textfieldFirstName: Locator;
  readonly textfieldLocality: Locator;
  readonly buttonNext: Locator;
  readonly buttonAdd: Locator;
  readonly buttonEditList: Locator;
  readonly buttonRemoveList: Locator;
  readonly buttonConfirmRemoveList: Locator;

  constructor(protected readonly page: Page) {
    this.buttonAddExistingCandidate = this.page.getByRole("link", {
      name: "Bestaande",
    });
    this.buttonAddNewCandidate = this.page.getByRole("link", {
      name: "Nieuwe",
    });
    this.buttonSearchExistingCandidate = this.page.getByLabel(
      "Zoek bestaande kandidaat",
    );
    this.textfieldInitials = this.page.getByLabel("Voorletters");
    this.textfieldLastName = this.page.getByLabel("Achternaam");
    this.textfieldFirstName = this.page.getByLabel("Roepnaam");
    this.textfieldLocality = this.page.getByLabel("Woonplaats");
    this.buttonNext = this.page.getByRole("button", { name: "Volgende" });
    this.buttonAdd = this.page.getByRole("button", { name: "Toevoegen" });
    this.buttonEditList = this.page.getByRole("link", { name: "Aanpassen" });
    this.buttonRemoveList = this.page.getByRole("button", {
      name: "Kandidatenlijst verwijderen",
    });
    this.buttonConfirmRemoveList = this.page.getByRole("button", {
      name: "Verwijderen",
      exact: true,
    });
  }

  async getCandidateLocator(candidateName: string) {
    return this.page.getByRole("cell", { name: candidateName });
  }

  async getDistrictLocator(districtName: string) {
    return this.page.getByRole("listitem", { name: districtName });
  }

  async addExistingCandidates(candidates: string[]) {
    for (const candidate of candidates) {
      await this.buttonAddExistingCandidate.click();

      // search first part of the name
      await this.buttonSearchExistingCandidate.pressSequentially(
        candidate.slice(0, 5),
      );

      await this.page
        .getByRole("row", { name: candidate })
        .getByRole("button")
        .click();
    }
  }

  async addNewCandidates(candidates: Candidate[]) {
    for (const candidate of candidates) {
      await this.buttonAddNewCandidate.click();
      await this.textfieldInitials.fill(candidate.initials);
      await this.textfieldLastName.fill(candidate.lastName);
      await this.textfieldFirstName.fill(candidate.firstName ?? "");
      await this.buttonNext.click();
      await this.textfieldLocality.fill(candidate.locality ?? "");
      await this.buttonAdd.click();
    }
  }

  async removeDistricts(districts: string[]) {
    await this.buttonEditList.click();
    for (const district of districts) {
      await this.page.getByRole("checkbox", { name: district }).uncheck();
    }
    await this.buttonNext.click();
  }

  async addDistricts(districts: string[]) {
    await this.buttonEditList.click();
    for (const district of districts) {
      await this.page.getByRole("checkbox", { name: district }).check();
    }
    await this.buttonNext.click();
  }

  async removeList() {
    await this.buttonEditList.click();
    await this.buttonRemoveList.click();
    await this.buttonConfirmRemoveList.click();
  }
}
