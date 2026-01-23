import { expect, Page } from "@playwright/test";
import { Candidate } from "../models/candidate";

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

    async addNewCandidates(candidates: Candidate[]) {
        for(var candidate of candidates) {
            await this.page.getByRole('link', { name: 'New' }).click();
            await this.page.getByLabel('Initials').fill(candidate.initials);
            await this.page.locator('input[name="last_name"]').fill(candidate.lastName);
            await this.page.getByLabel('First name').fill(candidate.firstName ?? '');
            await this.page.getByRole('button', {name: 'Save'}).click();
            await this.page.getByLabel('Locality').fill(candidate.locality ?? '');
            await this.page.getByRole('button', {name: 'Save'}).click(); 
        }      
    }

    async removeDistricts(districts: string[]) {
        await this.page.getByRole('link', { name: 'List details' }).click();
        for(var district of districts) {
            await this.page.getByRole('checkbox', { name: district }).uncheck();
        }
        await this.page.getByRole('button', { name: 'Save' }).click();
    }
    
    async addDistricts(districts: string[]) {
        await this.page.getByRole('link', { name: 'List details' }).click();
        for(var district of districts) {
            await this.page.getByRole('checkbox', { name: district }).check();
        }
        await this.page.getByRole('button', { name: 'Save' }).click();
    }
}
