// Highlight active navigation links based on the current URL.
export default function highlightActiveLinks() {
  document.querySelectorAll<HTMLAnchorElement>("a").forEach((link) => {
    // csb uses breadcrumbs, these don't need highlighting
    if (link.closest(".breadcrumbs")) return;

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
