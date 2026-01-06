# Definition of Done

## DoD PR

- Should be linked to one or more issues (merging the PR resolved the issue(s))
- Should be assigned (one person should be responsible for maintaining the PR)
- Unit tests for all added code
- Rust integration tests for all implemented user flows (tests that do not share code with the main crate)
- Explicitly test use-cases described in the issue
- If the PR fixes a bug: unit test to reproduce the bug
- All automatic checks should have passed
    - cargo fmt
    - cargo clippy
    - cargo test
    - Playwright test suite
    - Newlines at the end of text files
    - Sigrid
    - Codecov
    - Typescript linting
- The PR should specify how the changes can be tested by users
- All relevant code documentation has been added
- At least one review
    - Reviewer should read all changed code
    - The reviewer should audit the code quality
    - Should test the added or changes functionality either or both locally or on the test website
    - Should validate that the linked use-cases are implemented correctly

## DoD epic

- All issues should be finished and PR's merged
- All issues should meet the DoD issue level
- Playwright integration tests for all implemented user flows
- PO reviewed and tested all added functionality
- Internal knowledge sharing is done
- Test plan has been executed and passed

## DoD sprint release

- Create changelog and release notes
- Version number has been bumped
- Deploy to preview environment
- Smoke test performed on the preview environment (possibly the integration test suite)
- The integration tests should have run on all [target platforms](https://www.communicatierijk.nl/vakkennis/rijkswebsites/aanbevolen-richtlijnen/browsersupport)

## DoD major release

- All epics should meet the DoD epic level (do not deploy half implemented epics)
- The application has been audited / pen-tested
