// Enforce uppercase initials with dots and no spaces
window.addEventListener("load", () => {
  const initialsInputs: NodeListOf<HTMLInputElement> =
    document.querySelectorAll('input[name="initials"]');

  const checkbox: HTMLInputElement | null = document.querySelector(
    '.autoformat input[type="checkbox"]',
  );

  initialsInputs.forEach((input: HTMLInputElement) => {
    let lastKey: string | null = null;

    input.addEventListener("keydown", (event) => {
      lastKey = event.key;
    });

    input.addEventListener("input", () => {
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
    });
  });
});
