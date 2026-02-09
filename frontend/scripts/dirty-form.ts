function setupDirtyForms() {
  const forms = document.querySelectorAll("form");
  const dirtyForms = new Set<HTMLFormElement>();
  let isSubmitting = false;

  forms.forEach((form) => {
    const submitButton: HTMLButtonElement | null =
      form.querySelector("button.dirty-check");
    const anyFieldRequired = form.querySelector("[required]") !== null;

    if (!submitButton) {
      return;
    }

    const updateSubmitButtons = () => {
      if (dirtyForms.has(form) && (!anyFieldRequired || form.checkValidity())) {
        submitButton.disabled = false;
      } else {
        submitButton.disabled = true;
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

    updateSubmitButtons();
  });

  globalThis.addEventListener("beforeunload", (event) => {
    if (dirtyForms.size > 0 && !isSubmitting) {
      event.preventDefault();
    }
  });
}

if (typeof globalThis !== "undefined") {
  globalThis.addEventListener("load", setupDirtyForms);
}
