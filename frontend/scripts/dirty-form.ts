export default function setupDirtyForms() {
  const forms = document.querySelectorAll("form");
  const dirtyForms = new Set<HTMLFormElement>();
  let isSubmitting = false;

  forms.forEach((form) => {
    const submitButton: HTMLButtonElement | null = form.querySelector(
      "button[value='save']",
    );
    const anyFieldRequired = form.querySelector("[required]") !== null;

    if (submitButton === null) {
      return;
    }

    const canSubmit = () =>
      !submitButton.classList.contains("dirty-check") ||
      (dirtyForms.has(form) && (!anyFieldRequired || form.checkValidity()));

    const updateSubmitButtons = () => {
      if (canSubmit()) {
        submitButton.classList.remove("disabled");
      } else {
        submitButton.classList.add("disabled");
      }
    };

    const setDirty = () => {
      dirtyForms.add(form);
      updateSubmitButtons();
    };

    form.querySelectorAll("input, textarea, select").forEach((input) => {
      input.addEventListener("change", setDirty);
      input.addEventListener("input", setDirty);
      input.addEventListener("keydown", setDirty);
    });

    form.addEventListener("submit", () => {
      dirtyForms.delete(form);
      isSubmitting = true;
    });

    document.addEventListener("keydown", (event) => {
      if (event.key !== "Enter" || !event.shiftKey || event.isComposing) {
        return;
      }

      event.preventDefault();

      if (!canSubmit()) {
        updateSubmitButtons();
        return;
      }

      form.requestSubmit(submitButton);
    });

    updateSubmitButtons();
  });

  globalThis.addEventListener("beforeunload", (event) => {
    if (dirtyForms.size > 0 && !isSubmitting) {
      event.preventDefault();
    }
  });
}
