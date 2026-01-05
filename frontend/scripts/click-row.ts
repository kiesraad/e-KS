/// Make table rows clickable
window.addEventListener("load", () => {
  document.querySelectorAll("tr.clickable").forEach((row) => {
    row.addEventListener("click", (event) => {
      // skip if the click originated a cell with class drag-handle
      if ((event?.target as HTMLElement).closest(".drag-handle")) {
        return;
      }

      const link = row.querySelector("a,button") as
        | HTMLButtonElement
        | HTMLLinkElement
        | null;
      if (link) {
        link.click();
      }
    });
  });
});
