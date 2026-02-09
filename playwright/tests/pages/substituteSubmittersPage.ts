import type { Page } from "@playwright/test";
import type { ListSubmitter } from "../models/listSubmitter";

export class SubstituteSubmittersPage {
  private readonly page: Page;

  constructor(page: Page) {
    this.page = page;
  }

  getSubmitterLocator(lastName: string) {
    return this.page
      .locator("table.substitute-submitters-table")
      .getByRole("cell", { name: lastName });
  }

  async open() {
    await this.page.goto("/political-group/list-submitters");
  }

  async deleteExistingSubstituteSubmitters() {
    //takes all links from table and saves href attributes of each link in list
    const hrefs = await this.page
      .locator("table.substitute-submitters-table")
      .getByRole("link")
      .evaluateAll((links) => links.map((link) => link.getAttribute("href")));

    for (const href of hrefs) {
      if (href) {
        await this.page.goto(href);
        await this.page
          .getByRole("button", {
            name: "Vervanger voor herstel verzuimen verwijderen",
            exact: true,
          })
          .click();
        await this.page
          .getByRole("button", { name: "Verwijderen", exact: true })
          .click();
      }
    }
  }

  async addSubstituteSubmitter(listSubmitters: ListSubmitter[]) {
    for (const listSubmitter of listSubmitters) {
      await this.page
        .getByRole("link", { name: "Vervanger herstel verzuimen toevoegen" })
        .click();
      await this.page
        .getByRole("textbox", { name: "Voorletters *" })
        .fill(listSubmitter.initials);
      await this.page
        .getByRole("textbox", { name: "Voorvoegsel" })
        .fill(listSubmitter.lastNamePrefix ?? "");
      await this.page
        .getByRole("textbox", { name: "Achternaam *" })
        .pressSequentially(listSubmitter.lastName);

      await this.page.getByRole("button", { name: "Opslaan" }).click();
    }
  }

  async editListSubmitter() {
    await this.page
      .getByRole("cell", { name: "Vervanger herstel verzuimen toevoegen" })
      .click();
    await this.page.getByRole("textbox", { name: "Voorletters *" }).fill("A");
    await this.page.getByRole("textbox", { name: "Voorvoegsel" }).fill("de");
    await this.page
      .getByRole("textbox", { name: "Achternaam *" })
      .fill("Tester");
    await this.page.getByRole("button", { name: "Opslaan" }).click();
  }
}
