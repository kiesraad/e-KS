import { expect, test } from "@playwright/test";

import { setupClickCard } from "../../../frontend/scripts/click-card";

declare global {
  interface Window {
    recordClick: () => void;
  }
}

test.describe("click-card", () => {
  test("clicking a card triggers the link", async ({ page }) => {
    let clicked = false;
    await page.exposeFunction("recordClick", () => {
      clicked = true;
    });

    await page.setContent(`
      <div class="card card-clickable">
        <div class="content"><span id="card-target">Card</span></div>
        <a id="card-link" href="#">Details</a>
      </div>
    `);

    await page.evaluate(() => {
      const link = document.querySelector("#card-link");
      if (!link) {
        throw new Error("Missing link");
      }
      link.addEventListener("click", (event) => {
        event.preventDefault();
        window.recordClick();
      });
    });

    await page.evaluate(setupClickCard);

    await page.click("#card-target");

    await expect.poll(() => clicked).toBe(true);
  });
});
