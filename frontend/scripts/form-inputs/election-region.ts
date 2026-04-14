const FORM_SELECTOR = ".election-switch";

/**
 * Shows the region select matching the chosen election's region kind,
 * hides the others.
 */
function updateRegionVisibility(form: HTMLFormElement) {
  const select = form.querySelector<HTMLSelectElement>(
    'select[name="election"]',
  );
  const selected = select?.selectedOptions[0];
  const kind = selected?.dataset.regionKind ?? "";

  form.querySelectorAll<HTMLElement>(".region-select").forEach((el) => {
    el.style.display = el.dataset.regionKind === kind ? "" : "none";
  });
}

export default function electionRegion() {
  document.querySelectorAll<HTMLFormElement>(FORM_SELECTOR).forEach((form) => {
    updateRegionVisibility(form);

    const select = form.querySelector('select[name="election"]');
    select?.addEventListener("change", () => updateRegionVisibility(form));
  });
}
