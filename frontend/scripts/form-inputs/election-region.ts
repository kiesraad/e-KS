const FORM_SELECTOR = ".election-switch";

/**
 * Shows the region select matching the chosen election,
 * hides the others.
 */
function updateRegionVisibility(form: HTMLFormElement) {
  const select = form.querySelector<HTMLSelectElement>(
    'select[name="election"]',
  );
  const value = select?.value ?? "";

  form.querySelectorAll<HTMLElement>(".region-select").forEach((el) => {
    el.style.display = el.dataset.forElection === value ? "" : "none";
  });
}

export default function electionRegion() {
  document.querySelectorAll<HTMLFormElement>(FORM_SELECTOR).forEach((form) => {
    updateRegionVisibility(form);

    const select = form.querySelector('select[name="election"]');
    select?.addEventListener("change", () => updateRegionVisibility(form));
  });
}
