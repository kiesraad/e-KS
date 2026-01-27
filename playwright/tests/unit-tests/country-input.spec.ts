import { expect, test } from "@playwright/test";

import { countryCodeToFlagEmoji } from "../../../frontend/scripts/country-input";

test.describe("country-input", () => {
  test("country code to flag emoji", () => {
    expect(countryCodeToFlagEmoji("NL")).toBe("🇳🇱");
    expect(countryCodeToFlagEmoji("BE")).toBe("🇧🇪");
    expect(countryCodeToFlagEmoji("CW")).toBe("🇨🇼");
    expect(countryCodeToFlagEmoji("DE")).toBe("🇩🇪");
    expect(countryCodeToFlagEmoji("FR")).toBe("🇫🇷");
    expect(countryCodeToFlagEmoji("")).toBe("🌐");
    expect(countryCodeToFlagEmoji("X")).toBe("🌐");
    expect(countryCodeToFlagEmoji("XXX")).toBe("🌐");
  });
});
