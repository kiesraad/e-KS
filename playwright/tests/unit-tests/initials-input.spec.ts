import { expect, test } from "@playwright/test";

import initialsInput from "../../../frontend/scripts/form-inputs/initials-input";

const content = (value: string) => `
  <label class="autoformat"><input id="autoformat" type="checkbox" checked /></label>
  <input id="initials" class="initials-input" value="${value}" />
`;

const state = () => {
  const input = document.querySelector<HTMLInputElement>("#initials");
  const checkbox = document.querySelector<HTMLInputElement>("#autoformat");
  if (!input || !checkbox) {
    throw new Error("Missing initials input or checkbox");
  }
  return { value: input.value, checked: checkbox.checked };
};

const type = (value: string) => {
  const input = document.querySelector<HTMLInputElement>("#initials");
  if (!input) {
    throw new Error("Missing initials input");
  }
  input.value = value;
  input.dispatchEvent(new Event("input", { bubbles: true }));
};

test.describe("initials-input", () => {
  test("formats initials when autoformat is enabled", async ({ page }) => {
    await page.setContent(content(""));
    await page.evaluate(initialsInput);

    await page.evaluate(type, "ab");

    expect(await page.evaluate(state)).toEqual({
      value: "A.B.",
      checked: true,
    });
  });

  test("keeps letters with diacritics when autoformat is enabled", async ({
    page,
  }) => {
    await page.setContent(content(""));
    await page.evaluate(initialsInput);

    await page.evaluate(type, "éø");

    expect(await page.evaluate(state)).toEqual({
      value: "É.Ø.",
      checked: true,
    });
  });

  test("keeps autoformat enabled for values in the simple format", async ({
    page,
  }) => {
    await page.setContent(content("É.Ø."));
    await page.evaluate(initialsInput);

    expect(await page.evaluate(state)).toEqual({
      value: "É.Ø.",
      checked: true,
    });
  });

  test("disables autoformat when lower-case letters exist on load", async ({
    page,
  }) => {
    await page.setContent(content("ab"));
    await page.evaluate(initialsInput);

    expect(await page.evaluate(state)).toEqual({ value: "ab", checked: false });
  });

  // Initials of first names consisting of a single letter, which the BRP writes
  // without a dot and separated by a space, see
  // https://developer.rvig.nl/brp-api/personen/features/voorletters/
  for (const value of ["A", "S.Q", "J P"]) {
    test(`disables autoformat for initials '${value}' on load`, async ({
      page,
    }) => {
      await page.setContent(content(value));
      await page.evaluate(initialsInput);

      expect(await page.evaluate(state)).toEqual({ value, checked: false });
    });
  }
});
