export default function hashInput() {
  document
    .querySelectorAll<HTMLInputElement>("[data-hash-input]")
    .forEach((input) => {
      input.addEventListener("keydown", (e) => {
        if (
          e.key === "Backspace" &&
          !e.ctrlKey &&
          input.selectionStart === input.selectionEnd
        ) {
          const pos = input.selectionStart ?? 0;
          // when backspacing a space (somewhere in the middle of the hash)
          if (pos > 0 && input.value[pos - 1] === " ") {
            e.preventDefault();

            // also take out the character before the space
            input.value =
              input.value.slice(0, pos - 2) + input.value.slice(pos - 1);
            input.setSelectionRange(pos - 2, pos - 2);

            // then reformat
            input.dispatchEvent(new Event("input"));
          }
        }
      });

      input.addEventListener("input", () => {
        // remember cursor position without spaces
        const cursor = input.selectionStart ?? 0;
        const before = input.value
          .slice(0, cursor)
          .replace(/[^0-9a-fA-F]/g, "").length;

        // put spaces in between every 4 characters
        const raw = input.value
          .toUpperCase()
          .replace(/[^0-9A-F]/g, "")
          .slice(0, 64);
        input.value = raw.replace(/(.{4})/g, "$1 ").trim();

        // put the cursor back at the correct position with spaces
        const pos = before + Math.floor(before / 4);
        input.setSelectionRange(pos, pos);

        // show warning before submitting
        input.setCustomValidity(
          raw.length < 8 || raw.length % 2 === 1
            ? (input.dataset.invalidHashMessage ?? " ")
            : "",
        );
      });
    });
}
