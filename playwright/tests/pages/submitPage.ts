import type { Locator, Page } from "@playwright/test";

export class SubmitPage {
  readonly linkH1NLDownload: Locator;
  readonly linkH1FRDownload: Locator;
  readonly linkH31NLDownload: Locator;
  readonly linkH31FRDownload: Locator;
  readonly linkH4NLDownload: Locator;
  readonly linkH4FRDownload: Locator;
  readonly linkH9NLDownload: Locator;
  readonly linkH9FRDownload: Locator;

  constructor(protected readonly page: Page) {
    this.linkH1NLDownload = this.page.getByRole("link", {
      name: "H1 downloaden (Nederlands)",
    });
    this.linkH1FRDownload = this.page.getByRole("link", {
      name: "H1 downloaden (Fries)",
    });
    this.linkH31NLDownload = this.page.getByRole("link", {
      name: "H3-1 downloaden (Nederlands)",
    });
    this.linkH31FRDownload = this.page.getByRole("link", {
      name: "H3-1 downloaden (Fries)",
    });
    this.linkH4NLDownload = this.page.getByRole("link", {
      name: "H4 downloaden (Nederlands)",
    });
    this.linkH4FRDownload = this.page.getByRole("link", {
      name: "H4 downloaden (Fries)",
    });
    this.linkH9NLDownload = this.page.getByRole("link", {
      name: "H9 downloaden (Nederlands)",
    });
    this.linkH9FRDownload = this.page.getByRole("link", {
      name: "H9 downloaden (Fries)",
    });
  }
}
