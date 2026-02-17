export default function highlightActiveLinks() {
  document.querySelectorAll("a").forEach((link) => {
    const current = globalThis.location.pathname;
    const isMain = Boolean(link.closest("header"));

    // highlight active links or main menu items if the current path starts with the same segment
    if (
      current === link.pathname ||
      (isMain && link.pathname !== "/" && current.startsWith(link.pathname))
    ) {
      link.classList.add("active");
    }
  });
}
