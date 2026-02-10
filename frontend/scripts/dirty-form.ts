function setupDirtyForms() {
  const forms = document.querySelectorAll("form");
  const dirtyForms = new Set<HTMLFormElement>();
  let isSubmitting = false;

  forms.forEach((form) => {
    const submitButtons: NodeListOf<HTMLButtonElement> = form.querySelectorAll(
      "button[value='save'], button[value='next']",
    );
    const anyFieldRequired = form.querySelector("[required]") !== null;

    if (submitButtons.length === 0) {
      return;
    }

    const updateSubmitButtons = () => {
      if (dirtyForms.has(form) && (!anyFieldRequired || form.checkValidity())) {
        submitButtons.forEach((button) => {
          button.disabled = false;
        });
      } else {
        submitButtons.forEach((button) => {
          button.disabled = true;
        });
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
