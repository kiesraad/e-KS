export default function setupOverlay() {
  const overlay: HTMLElement | null = document.querySelector(".overlay");

  if (overlay) {
    const url = new URL(globalThis.location.href);
    url.searchParams.set("overlay", "true");
    globalThis.history.replaceState({}, "", url.toString());
  }
}
