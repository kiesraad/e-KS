import { test } from "@playwright/test";
import type { Candidate } from "./models/candidate";
import { CandidateListsOverviewPage } from "./pages/candidateListsOverviewPage";
import { ManageCandidateListPage } from "./pages/manageCandidateListPage";
import { SelectElectoralDistrictsPage } from "./pages/selectElectoralDistrictsPage";

test("add and delete candidate list", async ({ page }) => {
  const candidateListsOverviewPage = new CandidateListsOverviewPage(page);
  await candidateListsOverviewPage.open();
  await candidateListsOverviewPage.addList();

  const selectElectoralDistrictsPage = new SelectElectoralDistrictsPage(page);
  await selectElectoralDistrictsPage.selectDistricts([
    "Drenthe",
    "Groningen",
    "Overijssel",
  ]);

  const manageCandidateListPage = new ManageCandidateListPage(page);
  await manageCandidateListPage.addExistingCandidates([
    "Abdul Rahman, N.A. (Nadia)",
    "Ali, F.A. (Fatima)",
    "Alvarez, M.A. (Marco)",
  ]);

  const candidate: Candidate = {
    initials: "A",
    lastName: "Berg",
    firstName: "Anita",
    locality: "Utrecht",
  };
  const candidateTwo: Candidate = {
    initials: "B",
    lastName: "Beer",
  };
  await manageCandidateListPage.addNewCandidates([candidate, candidateTwo]);

  await manageCandidateListPage.removeList([
    "Drenthe",
    "Groningen",
    "Overijssel",
  ]);
});
