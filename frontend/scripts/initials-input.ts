// Enforce uppercase initials with dots and no spaces
export default function initialsInput() {
  const initialsInputs: NodeListOf<HTMLInputElement> =
    document.querySelectorAll("input.initials-input");

  const checkbox: HTMLInputElement | null = document.querySelector(
    '.autoformat input[type="checkbox"]',
  );

  initialsInputs.forEach((input: HTMLInputElement) => {
    let lastKey: string | null = null;

    // disable autoformatting if the field already contains a lowercase letter
    if (checkbox && /[a-z]/.test(input.value)) {
      checkbox.checked = false;
    }

    const format = () => {
      if (checkbox && !checkbox.checked) {
        return;
      }

      let initials = input.value.toUpperCase().replaceAll(/[^A-Z]/g, "");

      if (lastKey === "Backspace") {
        initials = initials.slice(0, -1);
        lastKey = null;
      }

      if (initials.length > 0) {
        input.value = `${initials.split("").join(".")}.`;
      } else {
        input.value = "";
      }
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
