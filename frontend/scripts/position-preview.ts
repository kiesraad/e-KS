window.addEventListener("load", () => {
    const preview = document.getElementById("position-preview");
    const input = document.getElementById("position");
    const table = document.getElementById("candidate-table");

    if (!input || !preview || !table) {
        return;
    }

    const rows = table.querySelectorAll<HTMLTableRowElement>("tbody tr");
    const data = Array.from(rows).map((row) => ({
            id: row.dataset.id,
            name: row.childNodes[1]?.innerHTML || "",
            locality: row.childNodes[2]?.innerHTML || "",
    }));

    console.log(data);
});
