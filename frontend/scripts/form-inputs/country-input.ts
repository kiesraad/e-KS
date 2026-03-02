// Enhance country code inputs with flag icons and keyboard navigation.
const COUNTRY_INPUT_SELECTOR = ".country-input";

type CountryInputElements = {
  textInput: HTMLInputElement;
  hint: HTMLSpanElement;
  flagIcon: HTMLSpanElement;
  list: HTMLElement;
  items: HTMLLIElement[];
  nlIndex: number;
};

/**
 * Collects and validates the required DOM elements for a country input control.
 * Returns null when required structure is missing.
 */
function getCountryInputElements(input: Element): CountryInputElements | null {
  const textInput = input.querySelector("input");
  const hint = input.parentNode?.querySelector(".hint") || null;
  const flagIcon = input.querySelector(".icon");
  const list = input.querySelector("ul");
  const items = Array.from(list?.querySelectorAll("li") || []);
  const nlIndex = items.findIndex((item) => item.dataset.country === "NL");

  if (!textInput || !flagIcon || !hint || !list || items.length === 0) {
    console.error("Country input is missing required elements");
    return null;
  }

  if (nlIndex === -1) {
    console.error("Country input list is missing NL country code");
    return null;
  }

  return {
    textInput,
    hint: hint as HTMLSpanElement,
    flagIcon: flagIcon as HTMLSpanElement,
    list,
    items,
    nlIndex,
  };
}

/**
 * Applies input configuration to improve typing behavior.
 */
function configureTextInput(textInput: HTMLInputElement) {
  // disable built-in browser autocomplete
  textInput.autocomplete = "off";
  textInput.autocapitalize = "characters";
}

/**
 * Shows the suggestion list.
 */
function showList(list: HTMLElement) {
  list.style.display = "block";
}

/**
 * Hides the suggestion list.
 */
function hideList(list: HTMLElement) {
  list.style.display = "none";
}

/**
 * Toggles the hint visibility based on the selected country.
 */
function updateHintVisibility(textInput: HTMLInputElement, hint: HTMLElement) {
  hint.style.display = textInput.value === "NL" ? "none" : "inline";
}

/**
 * Updates the flag icon to match the current country code value.
 */
function setFlagIcon(
  textInput: HTMLInputElement,
  items: HTMLLIElement[],
  flagIcon: HTMLSpanElement,
) {
  const inputValue = textInput.value.toUpperCase();
  const match = items.find((item) => item.dataset.country === inputValue);
  const icon: HTMLElement | null | undefined = match?.querySelector(".icon");
  flagIcon.innerText = icon?.innerText || "🌐";
}

/**
 * Ensures a default country code is set when the input is empty.
 */
function selectDefaultCountry(textInput: HTMLInputElement) {
  if (textInput.value === "") {
    textInput.value = "NL";
  }
}

/**
 * Applies the active state to the selected list item and scrolls it into view.
 */
function setActiveIndex(items: HTMLLIElement[], index: number) {
  items.forEach((item, itemIndex) => {
    item.classList.toggle("active", itemIndex === index);
  });
  items[index]?.scrollIntoView({ block: "center" });
}

/**
 * Finds the next active index based on the current input value.
 * Falls back to the NL index if no match is found.
 */
function findActiveIndex(
  textInput: HTMLInputElement,
  items: HTMLLIElement[],
  nlIndex: number,
) {
  const inputValue = textInput.value.toUpperCase();
  const matchIndex = items.findIndex(
    (item) =>
      inputValue.length > 0 && item.dataset.country?.startsWith(inputValue),
  );
  return matchIndex === -1 ? nlIndex : matchIndex;
}

/**
 * Wires up all behaviors for a single country input instance.
 */
function initCountryInput(elements: CountryInputElements) {
  const { textInput, hint, flagIcon, list, items, nlIndex } = elements;
  let active = 0;

  // hide hint
  hint.style.display = "none";

  configureTextInput(textInput);
  selectDefaultCountry(textInput);

  const updateSuggestions = () => {
    showList(list);
    active = findActiveIndex(textInput, items, nlIndex);
    setActiveIndex(items, active);
    setFlagIcon(textInput, items, flagIcon);
  };

  // render initial icon
  setFlagIcon(textInput, items, flagIcon);

  // show the suggestion list when focus on country code input
  textInput.addEventListener("focus", () => {
    textInput.select();
    updateSuggestions();
  });

  textInput.addEventListener("blur", () => {
    setTimeout(() => {
      hideList(list);
      updateHintVisibility(textInput, hint);
    }, 200);
  });

  textInput.addEventListener("input", updateSuggestions);

  items.forEach((item) => {
    item.addEventListener("click", () => {
      textInput.value = item.dataset.country || "";
      setFlagIcon(textInput, items, flagIcon);
      hideList(list);
    });
  });

  textInput.addEventListener("keydown", (event) => {
    if (event.key === "ArrowDown") {
      event.preventDefault();
      active = (active + 1) % items.length;
      setActiveIndex(items, active);
      return;
    }

    if (event.key === "ArrowUp") {
      event.preventDefault();
      active = (active - 1 + items.length) % items.length;
      setActiveIndex(items, active);
      return;
    }

    if (event.key === "Enter") {
      event.preventDefault();
      const selectedIndex = active % items.length;
      textInput.value = items[selectedIndex].dataset.country || "";
      setFlagIcon(textInput, items, flagIcon);
      hideList(list);
    }
  });
}

/**
 * Initializes all country input controls on the page.
 */
export default function countryCodeInput() {
  // Make flag icon match country code input
  const countryInputs = document.querySelectorAll(COUNTRY_INPUT_SELECTOR);

  countryInputs.forEach((input) => {
    const elements = getCountryInputElements(input);
    if (!elements) {
      return;
    }
    initCountryInput(elements);
  });
}
