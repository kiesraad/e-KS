const COUNTRY_INPUT_SELECTOR = ".country_input";

function getCountryInputs(): NodeListOf<HTMLInputElement> {
  return document.querySelectorAll(COUNTRY_INPUT_SELECTOR);
}

window.addEventListener("load", () => {
  const countryInputs = getCountryInputs();
  countryInputs.forEach((input) => {
    const textInput = input.querySelector("input");
    const flagIcon = input.querySelector(".icon");

    if (!textInput || !flagIcon) {
      console.error("Country input missing text or icon element");
      return;
    }

    const setFlagIcon = () => {
      let flag = "🌐";

      textInput.value = textInput.value.toUpperCase();

      if (/^[A-Z]{2}$/.test(textInput.value)) {
        const codePoints = [...textInput.value].map(
          (char) => 0x1f1e6 + char.charCodeAt(0) - 65,
        );

        flag = String.fromCodePoint(...codePoints);
      }

      flagIcon.textContent = flag;
    };

    textInput.addEventListener("input", setFlagIcon);
    setFlagIcon();
  });
});
