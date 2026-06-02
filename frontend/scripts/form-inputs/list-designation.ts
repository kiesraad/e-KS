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

function updateStepLabels(isCombined: boolean) {
  document
    .querySelectorAll<HTMLAnchorElement>("[data-label-combined]")
    .forEach((link) => {
      link.textContent = isCombined
        ? (link.dataset.labelCombined ?? "")
        : (link.dataset.label ?? "");
    });
}

export default function listDesignation() {
  const inputs = document.querySelectorAll<HTMLInputElement>(
    'input[name="list_designation_type"]',
  );

  inputs.forEach((input) => {
    input.addEventListener("change", () => {
      updateSkippedSteps(input.value === "blank" && input.checked);
      updateStepLabels(input.value === "combined" && input.checked);
    });
  });
}
