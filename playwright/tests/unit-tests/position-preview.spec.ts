import { expect, test } from "@playwright/test";

import { setupPositionPreview } from "../../../frontend/scripts/position-preview";

test.describe("position-preview", () => {
  test("updates row position, visibility, and fades", async ({ page }) => {
    await page.setContent(`
      <input id="position" value="3" />
      <div id="position-preview-container">
        <table id="position-preview">
          <tbody>
            <tr><td class="position-badge">1</td><td>Row 1</td></tr>
            <tr><td class="position-badge">2</td><td>Row 2</td></tr>
            <tr class="current"><td class="position-badge">3</td><td>Row 3</td></tr>
            <tr><td class="position-badge">4</td><td>Row 4</td></tr>
            <tr><td class="position-badge">5</td><td>Row 5</td></tr>
            <tr><td class="position-badge">6</td><td>Row 6</td></tr>
          </tbody>
        </table>
      </div>
    `);

    await page.evaluate(setupPositionPreview);

    const initial = await page.evaluate(() => {
      const container = document.getElementById("position-preview-container");
      const rows = Array.from(
        document.querySelectorAll<HTMLTableRowElement>(
          "#position-preview tbody tr",
        ),
      );

      return {
        currentIndex: rows.findIndex((row) =>
          row.classList.contains("current"),
        ),
        displays: rows.map((row) => row.style.display),
        badges: rows.map(
          (row) => row.querySelector(".position-badge")?.textContent,
        ),
        fadeTop: container?.classList.contains("fade-top") ?? false,
        fadeBottom: container?.classList.contains("fade-bottom") ?? false,
      };
    });

    expect(initial.currentIndex).toBe(2);
    expect(initial.fadeTop).toBe(false);
    expect(initial.fadeBottom).toBe(true);
    expect(initial.displays[0]).toBe("table-row");
    expect(initial.displays[5]).toBe("none");
    expect(initial.badges[2]).toBe("3");

    await page.evaluate(() => {
      const input = document.getElementById(
        "position",
      ) as HTMLInputElement | null;
      if (!input) {
        throw new Error("Missing position input");
      }
      input.value = "5";
      input.dispatchEvent(new Event("input", { bubbles: true }));
    });

    const updated = await page.evaluate(() => {
      const container = document.getElementById("position-preview-container");
      const rows = Array.from(
        document.querySelectorAll<HTMLTableRowElement>(
          "#position-preview tbody tr",
        ),
      );

      return {
        currentIndex: rows.findIndex((row) =>
          row.classList.contains("current"),
        ),
        displays: rows.map((row) => row.style.display),
        badges: rows.map(
          (row) => row.querySelector(".position-badge")?.textContent,
        ),
        fadeTop: container?.classList.contains("fade-top") ?? false,
        fadeBottom: container?.classList.contains("fade-bottom") ?? false,
      };
    });

    expect(updated.currentIndex).toBe(4);
    expect(updated.fadeTop).toBe(true);
    expect(updated.fadeBottom).toBe(false);
    expect(updated.displays[0]).toBe("none");
    expect(updated.displays[1]).toBe("none");
    expect(updated.displays[2]).toBe("table-row");
    expect(updated.badges[4]).toBe("5");
  });
});
