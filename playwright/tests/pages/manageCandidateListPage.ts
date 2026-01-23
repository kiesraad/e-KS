import { expect, Page } from "@playwright/test";

export class ManageCandidateListPage {
    
    private readonly page: Page;
            
    constructor(page: Page) {
       this.page = page;
        }

    async addExistingCandidates(candidates: string[]) {
        for(var candidate of candidates) {
            await this.page.getByRole('link', { name: 'Existing' }).click();
            await this.page.getByRole('row', { name: candidate }).getByRole('button').click();
            await expect(this.page.getByRole('cell', { name: candidate })).toBeVisible();
        }
    }
}
