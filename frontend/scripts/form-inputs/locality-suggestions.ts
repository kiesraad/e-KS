// Fetches and shows a single clickable locality suggestion under the input
// when the user's value doesn't exactly match a known Dutch locality. The
// suggestion is rendered by clicking the element with id `locality-suggestion-name`,
// which on click overwrites the input value and clears any warning state.
//
// The `/suggest` endpoint returns structured records. Field names mirror
// the upstream Dutch postal API:
//   wp  = "woonplaats" (place / city)
//   gm  = "gemeente"   (municipality)
//   pv  = "provincie"  (province)
//   had_suffix = true when the source disambiguated a duplicate place name
//                by appending a province suffix, so we must render "(pv)"
//                alongside the name to keep the display unambiguous.
//   alias      = alternate spelling/historical name that should also count
//                as an exact match even though it's not what we display.
type SuggestItem = {
  wp?: string;
  gm?: string;
  pv: string;
  had_suffix: boolean;
  alias?: string;
};

// Prefer woonplaats over gemeente; append province only when the source
// flagged this entry as a disambiguated duplicate.
function displayName(item: SuggestItem): string {
  const name = item.wp ?? item.gm ?? "";

  return item.had_suffix && item.pv ? `${name} (${item.pv})` : name;
}

// Aliases are a separate match path: a user typing an alias should be treated
// as having entered the canonical name, even though displayName() never returns
// the alias itself.
function aliasMatches(item: SuggestItem, query: string): boolean {
  return item.alias?.toLowerCase() === query.toLowerCase();
}

export default function localitySuggestions() {
  const input = document.getElementById("locality") as HTMLInputElement | null;
  const suggestion = document.getElementById("locality-suggestion");
  const suggestionName = document.getElementById("locality-suggestion-name");

  if (!input || !suggestion || !suggestionName) {
    return;
  }

  const field = input.closest(".form-field");

  // Accept the suggestion: copy the displayed name into the input, clear
  // the warning, and dispatch a change event so downstream listeners
  // (validators, address lookup) see the corrected value.
  suggestionName.addEventListener("click", () => {
    input.value = suggestionName.textContent ?? "";
    suggestion.classList.add("hidden");
    field?.classList.remove("warning");
    input.dispatchEvent(new Event("change", { bubbles: true }));
  });

  // The template opts in to municipality results by adding `with-municipalities`
  // to the suggestion element. Forms that only accept actual places leave it off.
  const withMunicipalities = suggestion.classList.contains(
    "with-municipalities",
  );

  // `warn` distinguishes "user is actively typing" (false) from "user has
  // committed by blurring, or the form just loaded with a prefilled value"
  // (true). While typing we never add a warning — only remove one if it
  // turns out the value is now valid — to avoid flashing red on every keystroke.
  const update = async (warn: boolean) => {
    const q = input.value;

    if (q.length === 0) {
      suggestion.classList.add("hidden");
      return;
    }

    // Below 3 characters the backend won't return useful results; treat
    // a too-short value as a warning on blur but stay silent while typing.
    if (q.length < 3) {
      suggestion.classList.add("hidden");
      if (warn) {
        field?.classList.add("warning");
      }
      return;
    }

    const url = withMunicipalities
      ? `/suggest?wp=${encodeURIComponent(q)}&municipalities=true&limit=1`
      : `/suggest?wp=${encodeURIComponent(q)}&municipalities=false&limit=1`;
    const res = await fetch(url);
    const suggestions: Array<SuggestItem> = await res.json();

    // A value is considered valid when it matches either the canonical
    // display form or a known alias of one of the returned suggestions.
    const exactMatch = suggestions.some(
      (s) =>
        displayName(s).toLowerCase() === q.toLowerCase() || aliasMatches(s, q),
    );
    if (warn) {
      field?.classList.toggle("warning", !exactMatch);
    } else if (exactMatch) {
      // Typing path: never add a warning here, but do clear one the moment
      // the value becomes valid so the user gets immediate positive feedback.
      field?.classList.remove("warning");
    }

    if (exactMatch) {
      suggestion.classList.add("hidden");
      return;
    }

    const first = suggestions[0];
    const name = first ? displayName(first) : "";
    if (!name) {
      suggestion.classList.add("hidden");
      return;
    }

    suggestionName.textContent = name;
    suggestion.classList.remove("hidden");
  };

  input.addEventListener("input", () => update(false));
  input.addEventListener("blur", () => update(true));
  // Initial pass: validate any prefilled value (e.g. when the form is
  // re-rendered with server-side state) and surface a warning if invalid.
  update(true);
}
