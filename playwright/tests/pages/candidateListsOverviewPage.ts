import { Page } from "@playwright/test";

export class CandidateListsOverviewPage {
        
    private readonly page: Page;
    
    constructor(page: Page) {
        this.page = page;
    }

    async open() {
        await this.page.goto('/candidate-lists');
    }

    async addList() {
        await this.page.getByRole('link', { name: 'Add list' }).click();
    }
}
