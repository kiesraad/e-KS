window.addEventListener("load", () => {
  const form = document.querySelector("form");
  const variantInputs: HTMLInputElement[] = Array.from(
    document.querySelectorAll("input[data-variant]"),
  );
  const variantRows: HTMLElement[] = Array.from(
    document.querySelectorAll("div[data-variant]"),
  );

  if (!form || variantInputs.length === 0 || variantRows.length === 0) {
    return;
  }

  // Function to update visible rows based on selected variant
  function update() {
    const variant = variantInputs.find((input) => input.checked)?.dataset
      .variant;

    variantRows.forEach((row) => {
      row.style.display = row.dataset.variant === variant ? "flex" : "none";
    });
  }

  // Listen for changes
  variantInputs.forEach((input) => {
    input.addEventListener("change", update);
  });

  // Initial update
  update();
});
