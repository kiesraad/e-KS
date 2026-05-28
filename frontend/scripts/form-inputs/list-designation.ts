function updateSkippedSteps(isBlank: boolean) {
  document
    .querySelectorAll<HTMLAnchorElement>("[data-skip-if-blank]")
    .forEach((link) => {
      if (isBlank) {
        link.removeAttribute("href");
        link.className = "disabled";
      } else {
        if (link.dataset.href) link.href = link.dataset.href;
        link.className = link.dataset.stateClass ?? "";
      }
    });
}

export default function listDesignation() {
  const inputs = document.querySelectorAll<HTMLInputElement>(
    'input[name="list_designation_type"]',
  );

  inputs.forEach((input) => {
    input.addEventListener("change", () => {
      updateSkippedSteps(input.value === "blank" && input.checked);
    });
  });
}
