export default function setupHintPopover() {
  const hint: HTMLDialogElement | null = document.querySelector("details.hint");
  const overlay: HTMLElement | null = document.querySelector(".overlay");

  if (hint && overlay) {
    overlay.addEventListener("click", (event: MouseEvent) => {
      // Anywhere on the overlay, except on the hint popover
      if (!hint.contains(event.target as Node) && hint.open) {
        event.stopImmediatePropagation();
        hint.open = false;
      }
    });

    globalThis.addEventListener("keydown", (event: KeyboardEvent) => {
      if (event.key === "Escape" && hint.open) {
        event.stopImmediatePropagation();
        hint.open = false;
      }
    });
  }
}
