window.addEventListener("load", () => {
  const search = document.getElementById("search") as HTMLInputElement | null;
  const table = document.getElementById(
    "add-candidate-table",
  ) as HTMLTableElement | null;

  if (!search || !table) {
    return;
  }

  search.addEventListener("input", (e) => {
    const searchValue = (e.target as HTMLInputElement).value.toLowerCase();
    const rows = table.querySelectorAll("tbody tr");

    rows.forEach((element: Element) => {
      const row = element as HTMLTableRowElement;
      const nameCell = row.querySelector("td:nth-child(2)");
      const localityCell = row.querySelector("td:nth-child(3)");

      if (nameCell && localityCell) {
        const nameText = nameCell.textContent?.toLowerCase() || "";
        const localityText = localityCell.textContent?.toLowerCase() || "";

        if (
          nameText.includes(searchValue) ||
          localityText.includes(searchValue)
        ) {
          row.style.display = "";
        } else {
          row.style.display = "none";
        }
      }
    });
  });
});
