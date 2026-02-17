// Highlight a table row when the `highlight` query param is present, then
// remove the param from the URL to avoid persistent state on refresh/share.
export default function highlightRow() {
  const url = new URL(globalThis.location.href);
  const personId = url.searchParams.get("highlight");
  const sticky = document.querySelector(".sticky-nav");

  if (!personId) {
    return;
  }

  // Match rows by data-id so deep links can target a specific person.
  const row = document.querySelector(`tr[data-id="${personId}"]`);

  // Clean the URL once we've captured the ID.
  url.searchParams.delete("highlight");
  globalThis.history.replaceState({}, "", url.toString());

  if (!row) {
    return;
  }

  // Apply the highlight and bring the row into view.
  row.classList.add("highlighted");
  row.scrollIntoView({ behavior: "auto", block: "center" });

  // Do not animate the sticky nav to avoid glitches on page load
  if (sticky) {
    sticky.classList.add("no-animation");
  }

  // Re-apply highlight after a short delay to ensure animation is visible.
  setTimeout(() => {
    row.classList.remove("highlighted");

    // After initial page render the sticky-nav can animate again
    if (sticky) {
      sticky.classList.remove("no-animation");
    }
  }, 2000);
}
