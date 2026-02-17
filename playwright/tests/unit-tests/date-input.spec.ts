import { expect, test } from "@playwright/test";

import { formatDateValue } from "../../../frontend/scripts/date-input";

test.describe("formatDateValue", () => {
  test("formats 8 digits into DD-MM-YYYY", () => {
    expect(formatDateValue("01022020")).toBe("01-02-2020");
  });

  test("strips unsupported characters", () => {
    expect(formatDateValue("ab1c2-3d4e2f0g7h8i9")).toBe("12-34-2078");
  });

  test("auto-pads day when a dash is typed after a single digit", () => {
    expect(formatDateValue("1-")).toBe("01-");
  });

  test("auto-pads day when the digit implies a day above 3", () => {
    expect(formatDateValue("4")).toBe("04-");
  });

  test("auto-pads month and keeps the second dash when month is implied", () => {
    expect(formatDateValue("12-3")).toBe("12-03-");
  });

  test("expands years starting with 3-9 to 19xx", () => {
    expect(formatDateValue("010134")).toBe("01-01-1934");
  });
});
