window.addEventListener("load", () => {
  const search = document.getElementById("search") as HTMLInputElement | null;
  const tableIds = ["add-candidate-table", "persons-table"];
  const table = tableIds
    .map((id) => document.getElementById(id))
    .find(Boolean) as HTMLTableElement | null;

  if (!search || !table) {
    return;
  }

  search.addEventListener("input", (e) => {
    const searchValue = (e.target as HTMLInputElement).value.toLowerCase();
    const rows = table.querySelectorAll("tbody tr");

    rows.forEach((element: Element) => {
      const row = element as HTMLTableRowElement;
      const rowText = row.textContent?.toLowerCase() || "";

      if (rowText.includes(searchValue)) {
        row.style.display = "";
      } else {
        row.style.display = "none";
      }
    });
  });
});
