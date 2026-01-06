// Enforce uppercase initials with dots and no spaces
window.addEventListener("load", () => {
  const initialsInputs: NodeListOf<HTMLInputElement> =
    document.querySelectorAll('input[name="initials"]');
  initialsInputs.forEach((input: HTMLInputElement) => {
    let lastKey: string | null = null;

    input.addEventListener("keydown", (event) => {
      lastKey = event.key;
    });

    input.addEventListener("input", () => {
      let initials = input.value.toUpperCase().replace(/[^A-Z]/g, "");

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
