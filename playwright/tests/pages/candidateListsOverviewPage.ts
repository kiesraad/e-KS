import { expect, Page } from "@playwright/test";

export class CandidateListsOverviewPage {
        
    private readonly page: Page;
    
    constructor(page: Page) {
        this.page = page;
    }

    async open() {
        await this.page.goto('/candidate-lists');
    }

    async addList() {
        await this.page.getByRole('main').getByRole('link', { name: 'Add list' }).click();
    }

    async manageList() {
        await this.page.getByRole('link', { name: 'Candidate list Electoral' }).first().click();
    }

    async managePersons() {
        await this.page.getByRole('heading', { name: 'All persons' }).click();
    }

    async checkDistricts(districts: string[]) {
        for(var district of districts) {
              await expect(this.page.getByRole('listitem', { name: district })).toHaveCount(0);
        }
    }
}
