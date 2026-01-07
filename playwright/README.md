# Playwright Tests

This directory contains end-to-end tests for the application using Playwright.

## Setup

From this `playwright` directory:

```bash
npm install
npx playwright install
```

`npx playwright install` downloads the required browsers (Chromium/Chrome, Firefox, WebKit/Safari, and Edge).

## Configuration

- Base URL: set `PLAYWRIGHT_BASE_URL` (defaults to `http://localhost:3000`).
- Tests live in `playwright/tests`.

## Running tests

Run only on Chrome:

```bash
npm run test:chrome
```

Run the full suite (Chrome, Firefox, Safari/WebKit, Edge):

```bash
npm run test:full
```

Other useful commands:

```bash
npm test
npm run test:ui
npm run report
```

## Notes

- If your app runs on a different port or URL, set `PLAYWRIGHT_BASE_URL`.
- Edge and Chrome use the system channels; if they are not installed locally, install them or remove the channel settings in `playwright.config.ts`.
