import { expect, test } from "@playwright/test";

import initialsInput from "../../../frontend/scripts/form-inputs/initials-input";

test.describe("initials-input", () => {
  test("formats initials when autoformat is enabled", async ({ page }) => {
    await page.setContent(`
      <label class="autoformat"><input id="autoformat" type="checkbox" checked /></label>
      <input id="initials" class="initials-input" value="" />
    `);

    await page.evaluate(initialsInput);

    await page.evaluate(() => {
      const input = document.querySelector<HTMLInputElement>("#initials");
      if (!input) {
        throw new Error("Missing initials input");
      }
      input.value = "ab";
      input.dispatchEvent(new Event("input", { bubbles: true }));
    });

    const value = await page.evaluate(() => {
      const input = document.querySelector<HTMLInputElement>("#initials");
      if (!input) {
        throw new Error("Missing initials input");
      }
      return input.value;
    });

    expect(value).toBe("A.B.");
  });

  test("disables autoformat when lower-case letters exist on load", async ({
    page,
  }) => {
    await page.setContent(`
      <label class="autoformat"><input id="autoformat" type="checkbox" checked /></label>
      <input id="initials" class="initials-input" value="ab" />
    `);

    await page.evaluate(initialsInput);

    const state = await page.evaluate(() => {
      const input = document.querySelector<HTMLInputElement>("#initials");
      const checkbox = document.querySelector<HTMLInputElement>("#autoformat");
      if (!input || !checkbox) {
        throw new Error("Missing initials input or checkbox");
      }
      return {
        value: input.value,
        checked: checkbox.checked,
      };
    });

    expect(state).toEqual({ value: "ab", checked: false });
  });
});
