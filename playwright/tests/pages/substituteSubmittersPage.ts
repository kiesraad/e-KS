import type { Page } from "@playwright/test";
import type { ListSubmitter } from "../models/listSubmitter";

export class SubstituteSubmittersPage {
  private readonly page: Page;

  constructor(page: Page) {
    this.page = page;
  }

  getSubmitterLocator(lastName: string) {
    return this.page.getByRole("link", { name: new RegExp(lastName) });
  }

  async open() {
    await this.page.goto("/political-group/list-submitters");
  }

  async deleteExistingSubstituteSubmitters() {
    //takes all links from table and saves href attributes of each link in list
    const hrefs = await this.page
      .locator(".substitute-list-submitters .person-block")
      .evaluateAll((links) => links.map((link) => link.getAttribute("href")));

    for (const href of hrefs) {
      if (href) {
        await this.page.goto(href);
        await this.page
          .getByRole("button", {
            name: "Vervanger voor het herstel van verzuimen verwijderen",
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
        .getByRole("link", { name: "Vervanger voor het herstel van verzuimen" })
        .click();
      await this.page
        .getByRole("textbox", { name: "Voorletters *" })
        .fill(listSubmitter.initials);
      await this.page
        .getByRole("textbox", { name: "Voorvoegsel" })
        .fill(listSubmitter.lastNamePrefix ?? "");
      await this.page
        .getByRole("textbox", { name: "Achternaam *" })
        .fill(listSubmitter.lastName);
      await this.page.locator("body").click();

      await this.page.getByRole("button", { name: "Opslaan" }).click();
    }
  }

  async editListSubmitter() {
    await this.page
      .getByRole("cell", { name: "Vervanger voor het herstel van verzuimen" })
      .click();
    await this.page.getByRole("textbox", { name: "Voorletters *" }).fill("A");
    await this.page.getByRole("textbox", { name: "Voorvoegsel" }).fill("de");
    await this.page
      .getByRole("textbox", { name: "Achternaam *" })
      .fill("Tester");
    await this.page.getByRole("button", { name: "Opslaan" }).click();
  }
}
