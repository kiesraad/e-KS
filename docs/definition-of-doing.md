# Definition of Doing

1. Pick an issue make sure to do the following:
   -  Drag it to the column "In Progress" on the [Sprint Board](https://github.com/orgs/kiesraad/projects/12/views/5);
   -  Assign the issue to yourself.
2. Design a solution for the issue:
   - Make notes inside the issue itself for traceability;
   - Look for a logical place in the code to fix the issue;
   - Consider refactoring if an elegant solution is hard to find;
   - Evaluate the Figma designs:
     - Are the designs technically feasible?
     - Are the design alternatives to consider (better UX or simpler to implement)?
3. Discuss the solution with colleague(s):
   - For complex technical issues, discuss with the tech lead (@marlonbaeten) or another developer;
   - For solution that don't fully comply with the Figma designs, discuss with the UI/UX designer (@Goran-GR).
4. Implement the solution:
   - Create a draft PR that is linked to the issue as soon as possible;
   - Commit and push often and early;
   - Keep the [DoD](/docs/definition-of-done.md) in mind when working on the issue.
5. Prepare the PR for review:
   - Remove draft status from the issue;
   - Pick a reviewer who is available and is knowledgeable about the affected parts in the code base;
   - Post a link to the PR in Mattermost ([#e-KS reviews](https://digilab.overheid.nl/chat/kiesraad/channels/e-ks-reviews)).
6. Iterate improvements and reviews until [DoD](/docs/definition-of-done.md) and acceptance criteria have been met.
7. Merge the PR into `main`.
