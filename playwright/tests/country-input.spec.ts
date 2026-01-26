import { expect, test } from "@playwright/test";

import { countryCodeToFlagEmoji } from "../../frontend/scripts/country-input";

test.describe("country-input", () => {
  test("country code to flag emoji", async () => {
    await expect(countryCodeToFlagEmoji("NL")).toBe("🇳🇱");
    await expect(countryCodeToFlagEmoji("BE")).toBe("🇧🇪");
    await expect(countryCodeToFlagEmoji("CW")).toBe("🇨🇼");
    await expect(countryCodeToFlagEmoji("DE")).toBe("🇩🇪");
    await expect(countryCodeToFlagEmoji("FR")).toBe("🇫🇷");
    await expect(countryCodeToFlagEmoji("")).toBe("🌐");
    await expect(countryCodeToFlagEmoji("X")).toBe("🌐");
    await expect(countryCodeToFlagEmoji("XXX")).toBe("🌐");
  });
});
