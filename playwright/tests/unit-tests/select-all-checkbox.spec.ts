import { expect, test } from "@playwright/test";

import setupSelectAllCheckbox from "../../../frontend/scripts/select-all-checkbox";

test.describe("select-all-checkbox", () => {
  test("initializes to indeterminate when some items are checked", async ({
    page,
  }) => {
    await page.setContent(`
      <div class="select-all-checkbox">
        <input id="select-all" type="checkbox" for-checklist="list" />
      </div>
      <div id="list">
        <label><input id="item-1" type="checkbox" checked /></label>
        <label><input id="item-2" type="checkbox" /></label>
      </div>
    `);

    await page.evaluate(setupSelectAllCheckbox);

    const state = await page.evaluate(() => {
      const selectAll = document.querySelector<HTMLInputElement>("#select-all");
      if (!selectAll) {
        throw new Error("Missing select all checkbox");
      }
      return {
        checked: selectAll.checked,
        indeterminate: selectAll.indeterminate,
        hasClass: selectAll.classList.contains("indeterminate"),
      };
    });

    expect(state).toEqual({
      checked: false,
      indeterminate: true,
      hasClass: true,
    });
  });

  test("checking the select-all checkbox updates the list", async ({
    page,
  }) => {
    await page.setContent(`
      <div class="select-all-checkbox">
        <input id="select-all" type="checkbox" for-checklist="list" />
      </div>
      <div id="list">
        <label><input id="item-1" type="checkbox" /></label>
        <label><input id="item-2" type="checkbox" /></label>
      </div>
    `);

    await page.evaluate(setupSelectAllCheckbox);
    await page.click("#select-all");

    const state = await page.evaluate(() => {
      const selectAll = document.querySelector<HTMLInputElement>("#select-all");
      const items = Array.from(
        document.querySelectorAll<HTMLInputElement>(
          "#list input[type=checkbox]",
        ),
      );
      if (!selectAll || items.length === 0) {
        throw new Error("Missing checkbox elements");
      }
      return {
        selectAllChecked: selectAll.checked,
        selectAllIndeterminate: selectAll.indeterminate,
        selectAllHasClass: selectAll.classList.contains("indeterminate"),
        itemsChecked: items.every((item) => item.checked),
      };
    });

    expect(state).toEqual({
      selectAllChecked: true,
      selectAllIndeterminate: false,
      selectAllHasClass: false,
      itemsChecked: true,
    });
  });

  test("changing a checklist item updates the select-all checkbox", async ({
    page,
  }) => {
    await page.setContent(`
      <div class="select-all-checkbox">
        <input id="select-all" type="checkbox" for-checklist="list" />
      </div>
      <div id="list">
        <label><input id="item-1" type="checkbox" checked /></label>
        <label><input id="item-2" type="checkbox" checked /></label>
      </div>
    `);

    await page.evaluate(setupSelectAllCheckbox);
    await page.click("#item-2");

    const state = await page.evaluate(() => {
      const selectAll = document.querySelector<HTMLInputElement>("#select-all");
      if (!selectAll) {
        throw new Error("Missing select all checkbox");
      }
      return {
        checked: selectAll.checked,
        indeterminate: selectAll.indeterminate,
        hasClass: selectAll.classList.contains("indeterminate"),
      };
    });

    expect(state).toEqual({
      checked: false,
      indeterminate: true,
      hasClass: true,
    });
  });
});
