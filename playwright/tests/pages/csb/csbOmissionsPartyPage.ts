import type { Locator, Page } from "@playwright/test";

export class csbOmissionsPartyPage {
  readonly linkAdd: Locator;
  readonly linkOverview: Locator;
  readonly buttonAuthoriseAppelation: Locator;
  readonly buttonAuthoriseCombination: Locator;
  readonly buttonAuthorisedAgent: Locator;
  readonly buttonAuthorisedAgentCombination: Locator;
  readonly buttonRegisterAppelation: Locator;
  readonly buttonRegisterCombination: Locator;
  readonly textfieldTitle: Locator;
  readonly textfieldDescription: Locator;
  readonly textfieldLetter: Locator;
  readonly checkboxRecoverable: Locator;
  readonly buttonAddAndClose: Locator;
  readonly linkClose: Locator;
  readonly buttonRemoveOmission: Locator;

  constructor(protected readonly page: Page) {
    this.linkAdd = this.page.getByRole("link", { name: "Verzuimen toevoegen" });
    this.linkOverview = this.page.getByRole("link", { name: "Overzicht" });
    this.buttonAuthoriseAppelation = this.page.getByRole("button", {
      name: "De machtiging aanduiding ontbreekt",
    });
    this.buttonAuthoriseCombination = this.page.getByRole("button", {
      name: "De machtiging samenvoeging ontbreekt",
    });
    this.buttonAuthorisedAgent = this.page.getByRole("button", {
      name: "De gemachtigde is niet geregistreerd",
    });
    this.buttonAuthorisedAgentCombination = this.page.getByRole("button", {
      name: "De gemachtigde(n) is/zijn niet geregistreerd",
    });
    this.buttonRegisterAppelation = this.page.getByRole("button", {
      name: "De aanduiding is niet geregistreerd",
    });
    this.buttonRegisterCombination = this.page.getByRole("button", {
      name: "De aanduiding(en) is/zijn niet geregistreerd",
    });
    this.textfieldTitle = this.page.getByRole("textbox", {
      name: "Titel Verzuim",
    });
    this.textfieldDescription = this.page.getByRole("textbox", {
      name: "I1 verzuim toevoegen",
    });
    this.textfieldLetter = this.page.getByRole("textbox", {
      name: "Verzuimbriefnotitie toevoegen",
    });
    this.checkboxRecoverable = this.page.getByRole("checkbox", {
      name: "Herstelbaar verzuim",
    });
    this.buttonAddAndClose = this.page.getByRole("button", {
      name: "Toevoegen en sluiten",
    });
    this.linkClose = this.page.getByRole("link", { name: "Sluiten" });
    this.buttonRemoveOmission = this.page.getByRole("button", {
      name: "Verwijderen",
    });
  }
}
