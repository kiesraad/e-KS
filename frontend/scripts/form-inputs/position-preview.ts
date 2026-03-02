// Preview candidate list positions relative to the current selection.
export default function setupPositionPreview() {
  const container = document.getElementById("position-preview-container");
  const preview = document.getElementById("position-preview");
  const input = document.getElementById("position");

  if (!input || !preview || !container) {
    return;
  }

  const originalPosition = Number.parseInt(
    (input as HTMLInputElement).value,
    10,
  );
  const tbody = preview.querySelector("tbody");
  const current = preview.querySelector("tr.current");

  if (!current || !tbody) {
    return;
  }

  const rows = tbody.getElementsByTagName("tr");
  const total = rows.length;

  const updatePreview = () => {
    let position = Number.parseInt((input as HTMLInputElement).value, 10);
    position = Math.min(Math.max(1, position), total) || originalPosition;

    current.remove();
    tbody.insertBefore(current, rows[position - 1]);

    // update position numbers
    for (let i = 0; i < total; i++) {
      const row = rows[i];

      if (i > position + 1 || i < position - 3) {
        row.style.display = "none";
      } else {
        row.style.display = "table-row";
        const cell = row.querySelector(".position-badge");
        if (cell) {
          cell.textContent = (i + 1).toString();
        }
      }
    }

    container.classList.toggle("fade-top", position > 3);
    container.classList.toggle("fade-bottom", position < total - 2);
  };

  input.addEventListener("input", () => {
    container.classList.add("visible");
    updatePreview();
  });

  updatePreview();
}
