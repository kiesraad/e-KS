import path from "node:path";
import type { Locator, Page } from "@playwright/test";

export class CsvImportExportPage {
  readonly buttonDownload: Locator;
  readonly buttonUpload: Locator;
  readonly buttonDownloadTemplate: Locator;
  readonly buttonContinue: Locator;
  readonly buttonCancel: Locator;
  readonly buttonClose: Locator;
  readonly headerImport: Locator;
  readonly textFailure: Locator;

  constructor(protected readonly page: Page) {
    this.buttonDownload = this.page.getByRole("link", {
      name: "Download CSV bestand",
    });
    this.buttonUpload = this.page.getByRole("button", {
      name: "Upload CSV bestand",
    });
    this.buttonDownloadTemplate = this.page.getByRole("button", {
      name: "Download CSV sjabloon",
    });
    this.buttonContinue = this.page.getByRole("button", { name: "Doorgaan" });
    this.buttonCancel = this.page.getByRole("button", { name: "Annuleren" });
    this.buttonClose = this.page.getByRole("link", { name: "Sluiten" });
    this.headerImport = this.page.getByRole("heading", {
      name: "Import en export kandidatenlijst",
    });
    this.textFailure = this.page.getByText("Importeren niet gelukt");
  }

  async getValidationErrors(validationError: string) {
    return this.page.getByText(`Controleer veld ${validationError}`);
  }

  async uploadCsvFile(filePath: string) {
    await this.buttonUpload.click();
    const [fileChooser] = await Promise.all([
      this.page.waitForEvent("filechooser"),
      this.buttonContinue.click(),
    ]);
    await fileChooser.setFiles(path.join(__dirname, "../testdata", filePath));
  }
}
