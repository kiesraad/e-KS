// Mark overlay state in the URL and support closing via Escape.
export default function setupOverlay() {
  const overlay: HTMLElement | null = document.querySelector(".overlay");

  if (overlay) {
    const url = new URL(globalThis.location.href);
    url.searchParams.set("overlay", "true");
    globalThis.history.replaceState({}, "", url.toString());

    const handleKeydown = (event: KeyboardEvent) => {
      if (event.key !== "Escape") {
        return;
      }

      const closeLink =
        overlay.querySelector<HTMLAnchorElement>(".close-overlay");

      if (closeLink) {
        event.preventDefault();
        window.location.href = closeLink.href;
      }
    };

    globalThis.addEventListener("keydown", handleKeydown);
  }
}
