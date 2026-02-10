const COUNTRY_INPUT_SELECTOR = ".country-input";

function applyCountrySuggestions() {
  // Make flag icon match country code input
  const countryInputs = document.querySelectorAll(COUNTRY_INPUT_SELECTOR);

  countryInputs.forEach((input) => {
    const textInput = input.querySelector("input");
    const flagIcon: HTMLSpanElement | null = input.querySelector(".icon");
    const list = input.querySelector("ul") as HTMLElement;
    const items = Array.from(list?.querySelectorAll("li") || []);
    const countries = items.map((i) => i.dataset.country) as string[];
    const flags = items
      .map((i) => i.firstChild as HTMLSpanElement)
      .map((span) => span.innerText);
    let active = 0;
    let suggestions: number[] = [];

    if (!textInput || !flagIcon) {
      console.error("Country input missing text or icon element");
      return;
    }

    // disable built-in browser autocomplete
    textInput.autocomplete = "off";
    textInput.autocapitalize = "characters";

    const setFlagIcon = () => {
      const inputValue = textInput.value.toUpperCase();
      const flag = countries.indexOf(inputValue);
      flagIcon.innerText = flags[flag] || "🌐";
    };

    const updateSuggestions = () => {
      // show the list of suggestions
      list.style.display = "block";
      const newValue = textInput.value.toUpperCase();

      // suggestions is a list of item indexes
      suggestions = [];
      // currently highlighted suggestion
      active = 0;

      countries.forEach((country, index) => {
        items[index].classList.remove("active");
        if (country.startsWith(newValue)) {
          items[index].style.display = "block";
          suggestions.push(index);
          if (suggestions.length === 1) {
            items[index].classList.add("active");
            items[index].scrollIntoView();
          }
        } else {
          items[index].style.display = "none";
        }
      });

      setFlagIcon();
    };

    // Select NL by default if no country is selected
    if (textInput.value === "") {
      textInput.value = "NL";
    }

    // render initial icon
    setFlagIcon();

    // show the suggestion list when focus on country code input
    textInput.addEventListener("focus", () => {
      updateSuggestions();
      list.style.display = "block";
    });

    textInput.addEventListener("blur", () => {
      setTimeout(() => {
        list.style.display = "none";
      }, 200);
    });

    textInput.addEventListener("input", updateSuggestions);

    items.forEach((item) => {
      item.addEventListener("click", () => {
        textInput.value = item.dataset.country || "";
        setFlagIcon();
        list.style.display = "none";
      });
    });

    textInput.addEventListener("keydown", (event) => {
      if (event.key === "ArrowDown") {
        event.preventDefault();
        active = (active + 1) % suggestions.length;
        items.forEach((item, index) => {
          item.classList.toggle("active", suggestions[active] === index);
        });
        items[suggestions[active]].scrollIntoView({ block: "nearest" });
      } else if (event.key === "ArrowUp") {
        event.preventDefault();
        active = (active - 1 + suggestions.length) % suggestions.length;
        items.forEach((item, index) => {
          item.classList.toggle("active", suggestions[active] === index);
        });
        items[suggestions[active]].scrollIntoView({ block: "nearest" });
      } else if (event.key === "Enter") {
        event.preventDefault();
        if (suggestions.length > 0) {
          const selectedIndex = suggestions[active];
          textInput.value = countries[selectedIndex];
          setFlagIcon();
          list.style.display = "none";
        }
      }
    });
  });
}

if (typeof window !== "undefined") {
  window.addEventListener("load", () => {
    applyCountrySuggestions();
  });
}
