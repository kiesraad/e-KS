import { expect, test } from "@playwright/test";

import setupSortable from "../../../frontend/scripts/sortable";

declare global {
  interface Window {
    recordFetch: (
      input: string,
      init: { method?: string; body?: string },
    ) => void;
  }
}

test.describe("sortable", () => {
  test("dragging a row updates order and sends fetch", async ({ page }) => {
    const fetchCalls: Array<{
      input: string;
      init: { method?: string; body?: string };
    }> = [];
    await page.exposeFunction(
      "recordFetch",
      (input: string, init: { method?: string; body?: string }) => {
        fetchCalls.push({ input, init });
      },
    );

    await page.setContent(`
      <table class="sortable" data-sortable-update-url="/reorder">
        <tbody>
          <tr data-id="1" id="row-1">
            <td class="drag-handle">::</td>
            <td><span class="position-badge">1</span></td>
            <td>Row 1</td>
          </tr>
          <tr data-id="2" id="row-2">
            <td class="drag-handle">::</td>
            <td><span class="position-badge">2</span></td>
            <td>Row 2</td>
          </tr>
          <tr data-id="3" id="row-3">
            <td class="drag-handle">::</td>
            <td><span class="position-badge">3</span></td>
            <td>Row 3</td>
          </tr>
        </tbody>
      </table>
    `);

    await page.evaluate(() => {
      window.fetch = (input: RequestInfo | URL, init?: RequestInit) => {
        const url =
          typeof input === "string" || input instanceof URL
            ? input.toString()
            : input.url;

        const payload = {
          input: url,
          init: {
            method: init?.method,
            body: typeof init?.body === "string" ? init.body : undefined,
          },
        };

        window.recordFetch(payload.input, payload.init);

        return Promise.resolve(new Response("", { status: 200 }));
      };
    });

    await page.evaluate(() => {
      const rows = Array.from(
        document.querySelectorAll<HTMLTableRowElement>(
          "table.sortable tbody tr",
        ),
      );
      rows.forEach((row, index) => {
        const top = index * 20;
        const height = 20;
        row.getBoundingClientRect = () =>
          ({
            x: 0,
            y: top,
            top,
            left: 0,
            bottom: top + height,
            right: 200,
            width: 200,
            height,
          }) as DOMRect;
      });
    });

    await page.evaluate(setupSortable);

    const dragDistance = 30;
    await page.evaluate((distance) => {
      const handle = document.querySelector<HTMLTableCellElement>(
        "#row-1 td.drag-handle",
      );
      if (!handle) {
        throw new Error("Missing drag handle");
      }
      handle.dispatchEvent(
        new MouseEvent("mousedown", { bubbles: true, clientY: 10 }),
      );
      window.dispatchEvent(
        new MouseEvent("mousemove", { bubbles: true, clientY: 10 + distance }),
      );
    }, dragDistance);

    await page.waitForFunction(() => {
      const row = document.querySelector<HTMLTableRowElement>("#row-1");
      return row?.style.transform.includes("translate") ?? false;
    });

    await page.evaluate((distance) => {
      window.dispatchEvent(
        new MouseEvent("mouseup", { bubbles: true, clientY: 10 + distance }),
      );
    }, dragDistance);

    await expect.poll(() => fetchCalls.length).toBe(1);

    const order = await page.evaluate(() =>
      Array.from(
        document.querySelectorAll<HTMLTableRowElement>(
          "table.sortable tbody tr",
        ),
      ).map((row) => row.dataset.id),
    );
    expect(order).toEqual(["2", "1", "3"]);

    const [fetchCall] = fetchCalls;
    expect(fetchCall?.input).toBe("/reorder");
    expect(fetchCall?.init?.method).toBe("POST");
    expect(JSON.parse(fetchCall?.init?.body || "")).toEqual({
      person_ids: ["2", "1", "3"],
    });
  });
});
