/// Make table rows clickable
export default function setupClickRow() {
  document.querySelectorAll("tr.clickable").forEach((row) => {
    row.addEventListener("click", (event) => {
      // skip if the click originated a cell with class drag-handle
      if (
        event?.target instanceof HTMLElement &&
        (event.target.closest(".drag-handle") ||
          event.target.classList.contains("flash-success"))
      ) {
        return;
      }

      const links = row.querySelectorAll<HTMLAnchorElement | HTMLButtonElement>(
        "a,button",
      );
      const link = links.item(links.length - 1);

      // ignore clicks on links or buttons that are not the last one in the row
      if (
        event?.target instanceof HTMLElement &&
        event.target.closest("a,button") &&
        event.target !== link
      ) {
        return;
      }

      if (link) {
        link.click();
      }
    });
  });
}
