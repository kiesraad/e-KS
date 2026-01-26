const COUNTRY_INPUT_SELECTOR = ".country_input";
const COUNTRY_OPTION_SELECTOR = "#countries option";

function getCountryInputs(): NodeListOf<HTMLInputElement> {
  return document.querySelectorAll(COUNTRY_INPUT_SELECTOR);
}

function getCountryOptions(): NodeListOf<HTMLOptionElement> {
  return document.querySelectorAll(COUNTRY_OPTION_SELECTOR);
}

export const countryCodeToFlagEmoji = (countryCode: string) => {
  if (/^[A-Z]{2}$/.test(countryCode)) {
    const codePoints = [...countryCode].map(
      (char) => 0x1f1e6 + char.charCodeAt(0) - 65,
    );

    return String.fromCodePoint(...codePoints);
  } else {
    return "🌐";
  }
};

if (typeof window !== "undefined") {
  window.addEventListener("load", () => {
    // Make flag icon match country code input
    const countryInputs = getCountryInputs();
    countryInputs.forEach((input) => {
      const textInput = input.querySelector("input");
      const flagIcon = input.querySelector(".icon");

      if (!textInput || !flagIcon) {
        console.error("Country input missing text or icon element");
        return;
      }

      const setFlagIcon = () => {
        textInput.value = textInput.value.toUpperCase();
        flagIcon.textContent = countryCodeToFlagEmoji(textInput.value);
      };

      textInput.addEventListener("input", setFlagIcon);
      setFlagIcon();
    });

    // Add country flags to options
    const options = getCountryOptions();
    options.forEach((option) => {
      option.innerText = `${countryCodeToFlagEmoji(option.value)} ${option.value}`;
    });
  });
}
