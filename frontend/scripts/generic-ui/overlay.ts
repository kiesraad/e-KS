// Mark overlay state in the URL and support closing via Escape.
export default function setupOverlay() {
  const overlay: HTMLElement | null = document.querySelector(".overlay");
  const backdrop: HTMLElement | null =
    document.querySelector(".overlay-backdrop");

  if (overlay && backdrop) {
    const url = new URL(globalThis.location.href);
    url.searchParams.set("overlay", "true");
    globalThis.history.replaceState({}, "", url.toString());

    // Close the overlay (click the close button) when the backdrop is clicked or Escape is pressed.
    const close = () => {
      const closeLink =
        overlay.querySelector<HTMLAnchorElement>(".close-overlay");

      if (closeLink) {
        globalThis.location.href = closeLink.href;
      }
    };

    let pressedOnBackdrop = false;

    backdrop.addEventListener("pointerdown", (event: PointerEvent) => {
      pressedOnBackdrop = event.target === backdrop;
    });

    backdrop.addEventListener("pointerup", (event: PointerEvent) => {
      if (pressedOnBackdrop && event.target === backdrop) {
        close();
      }
      pressedOnBackdrop = false;
    });

    globalThis.addEventListener("keydown", (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        close();
      }
    });
  }
}
