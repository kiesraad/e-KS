// Intercept step navigation links within an overlay form.
// When the form has unsaved changes (is "dirty"), submitting saves the changes
// and redirects to the target step. When the form is clean, the link navigates
// directly without submitting, avoiding unnecessary server round-trips.
//
// Dirty state is detected by comparing a snapshot of the initial form values
// (taken on setup) against the current values at the time of the click.

/** Serialize all form values into a comparable string. */
function formSnapshot(form: HTMLFormElement): string {
  const entries = [...new FormData(form).entries()] as [string, string][];
  return new URLSearchParams(entries).toString();
}

export default function setupStepNav() {
  const formElement = document.querySelector<HTMLFormElement>(
    "form.overlay, form.page-form",
  );

  if (!formElement) {
    return;
  }

  const initial = formSnapshot(formElement);
  const currentUrl = new URL(globalThis.location.href);
  const redirectTo = currentUrl.searchParams.get("redirect_to");

  formElement
    .querySelectorAll<HTMLAnchorElement>(".steps-nav a")
    .forEach((link) => {
      // For clean navigation: preserve redirect_to in the link href so it
      // survives the step transition.
      if (redirectTo?.startsWith("/")) {
        const linkUrl = new URL(link.href);
        linkUrl.searchParams.set("redirect_to", redirectTo);
        link.href = linkUrl.toString();
      }

      link.addEventListener("click", (event) => {
        const submitBtn = formElement.querySelector<HTMLButtonElement>(
          "button[value='save']",
        );

        if (!submitBtn) {
          return;
        }

        // Form is clean — let the browser follow the link normally.
        if (formSnapshot(formElement) === initial) {
          return;
        }

        // The form has unsaved changes: submit it and redirect to the target
        // step. Preserve the original redirect_to so the close button on the
        // next step still returns to the right page.
        event.preventDefault();
        const action = new URL(globalThis.location.href);
        let targetStep = link.pathname;
        if (redirectTo?.startsWith("/")) {
          targetStep += `?redirect_to=${encodeURIComponent(redirectTo)}`;
        }
        action.searchParams.set("redirect_to", targetStep);
        formElement.action = action.toString();
        formElement.requestSubmit(submitBtn);
      });
    });
}
