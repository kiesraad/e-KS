// Enforce the simple initials format: upper-case letters, each followed by a
// dot and no spaces (e.g. `A.B.`). Letters with diacritics are allowed, mirroring
// the characters the BRP allows. Initials that cannot be written in this format
// (a first name of a single letter is abbreviated without a dot, and separated
// from the next initial by a space, e.g. `S.Q` or `J P`) need autoformatting to
// be turned off.
export default function initialsInput() {
  // The letters allowed by the BRP: ASCII letters and `À-ž`, without the
  // multiplication and division signs in that range
  const letter = /[A-Za-zÀ-ÖØ-öø-ž]/;

  const formatInitials = (value: string): string => {
    const letters = Array.from(value.toUpperCase()).filter((character) =>
      letter.test(character),
    );

    return letters.length > 0 ? `${letters.join(".")}.` : "";
  };

  const initialsInputs: NodeListOf<HTMLInputElement> =
    document.querySelectorAll("input.initials-input");

  const checkbox: HTMLInputElement | null = document.querySelector(
    '.autoformat input[type="checkbox"]',
  );

  initialsInputs.forEach((input: HTMLInputElement) => {
    let lastKey: string | null = null;

    // disable autoformatting if the field contains initials that the
    // autoformatter would rewrite, so that its value is left untouched
    if (checkbox && formatInitials(input.value) !== input.value) {
      checkbox.checked = false;
    }

    const format = () => {
      if (checkbox && !checkbox.checked) {
        return;
      }

      let initials = formatInitials(input.value);

      if (lastKey === "Backspace") {
        // the trailing dot was deleted, so remove the initial it belongs to
        initials = formatInitials(initials.slice(0, -2));
        lastKey = null;
      }

      input.value = initials;
    };

    input.addEventListener("keydown", (event) => {
      lastKey = event.key;
    });

    input.addEventListener("input", format);

    if (checkbox !== null) {
      checkbox.addEventListener("click", format);
    }

    format();
  });
}
