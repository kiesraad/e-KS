function setupDirtyForms() {
  const forms = document.querySelectorAll("form");
  const dirtyForms = new Set<HTMLFormElement>();
  let isSubmitting = false;

  forms.forEach((form) => {
    const submitButton: HTMLButtonElement | null = form.querySelector(
      'button[type="submit"]',
    );
    const anyFieldRequired = form.querySelector("[required]") !== null;

    if (!submitButton || !anyFieldRequired) {
      return;
    }

    let isDirty = false;

    const updateSubmitButtons = () => {
      if (isDirty && form.checkValidity()) {
        submitButton.disabled = false;
      } else {
        submitButton.disabled = true;
      }
    };

    form.addEventListener("input", () => {
      isDirty = true;
      dirtyForms.add(form);
      updateSubmitButtons();
    });

    form.addEventListener("change", () => {
      isDirty = true;
      dirtyForms.add(form);
      updateSubmitButtons();
    });

    form.addEventListener("submit", () => {
      isDirty = false;
      dirtyForms.delete(form);
      isSubmitting = true;
    });

    updateSubmitButtons();
  });

  window.addEventListener("beforeunload", (event) => {
    if (dirtyForms.size > 0 && !isSubmitting) {
      event.preventDefault();
    }
  });
}

if (typeof window !== "undefined") {
  window.addEventListener("load", setupDirtyForms);
}
