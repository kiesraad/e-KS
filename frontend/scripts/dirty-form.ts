function setupDirtyForms() {
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

    const updateSubmitButtons = () => {
      if (submitButton.classList.contains("dirty-check")) {
        if (
          dirtyForms.has(form) &&
          (!anyFieldRequired || form.checkValidity())
        ) {
          submitButton.classList.remove("disabled");
        } else {
          submitButton.classList.add("disabled");
        }
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
