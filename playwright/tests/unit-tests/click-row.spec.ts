import { expect, test } from "@playwright/test";

import setupClickRow from "../../../frontend/scripts/click-row";

declare global {
  interface Window {
    recordClick: () => void;
  }
}

test.describe("click-row", () => {
  test("clicking a row triggers the link", async ({ page }) => {
    let clicked = false;
    await page.exposeFunction("recordClick", () => {
      clicked = true;
    });

    await page.setContent(`
      <table>
        <tr class="clickable">
          <td class="non-link"><span id="row-target">Row</span></td>
          <td><a id="row-link" href="#">Details</a></td>
        </tr>
      </table>
    `);

    await page.evaluate(() => {
      const link = document.querySelector("#row-link");
      if (!link) {
        throw new Error("Missing link");
      }
      link.addEventListener("click", (event) => {
        event.preventDefault();
        window.recordClick();
      });
    });

    await page.evaluate(setupClickRow);

    await page.click("#row-target");

    await expect.poll(() => clicked).toBe(true);
  });

  test("clicking a drag handle does not trigger the link", async ({ page }) => {
    let clicked = false;
    await page.exposeFunction("recordClick", () => {
      clicked = true;
    });

    await page.setContent(`
      <table>
        <tr class="clickable">
          <td><span class="drag-handle" id="drag-handle">Drag</span></td>
          <td><a id="row-link" href="#">Details</a></td>
        </tr>
      </table>
    `);

    await page.evaluate(() => {
      const link = document.querySelector("#row-link");
      if (!link) {
        throw new Error("Missing link");
      }
      link.addEventListener("click", (event) => {
        event.preventDefault();
        window.recordClick();
      });
    });

    await page.evaluate(setupClickRow);

    await page.click("#drag-handle");

    await page.waitForTimeout(50);
    expect(clicked).toBe(false);
  });
});
