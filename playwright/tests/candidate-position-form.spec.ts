import { expect } from "@playwright/test";
import { test } from "./fixtures.ts";
import { CandidateListsOverviewPage } from "./pages/candidateListsOverviewPage";
import { ManageCandidateListPage } from "./pages/manageCandidateListPage";
import { EditListDetailsPage } from "./pages/editListDetailsPage";

test.describe("candidate position form", () => {
  test("pressing Enter in the position field saves instead of removing the candidate", async ({
    login: page,
  }) => {
    await page.goto("/candidate-lists");
    await new CandidateListsOverviewPage(page).buttonAddList.click();
    await new EditListDetailsPage(page).addDistricts(["Limburg"]);

    const managePage = new ManageCandidateListPage(page);
    await managePage.addExistingCandidates(["Nagelhout", "Meerman"]);

    // Open the position form for the first candidate
    await page
      .getByRole("row", { name: "Nagelhout" })
      .getByRole("link", { name: "Bewerken" })
      .click();

    // Press Enter in the position input — this should trigger save, not remove
    await page.getByLabel("Positie").press("Enter");

    // The save action redirects to the personal details step (/update/{person_id}),
    // while the remove action would redirect back to the candidate list view
    await expect(page).toHaveURL(/\/update\//);
  });
});
