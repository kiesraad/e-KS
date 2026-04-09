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
  const form = document.querySelector<HTMLFormElement>("form.overlay");
  if (!form) return;

  const initial = formSnapshot(form);

  form.querySelectorAll<HTMLAnchorElement>(".steps-nav a").forEach((link) => {
    link.addEventListener("click", (event) => {
      const submitBtn = form.querySelector<HTMLButtonElement>(
        "button[value='save']",
      );

      if (!submitBtn) {
        return;
      }

      // Form is clean — let the browser follow the link normally.
      if (formSnapshot(form) === initial) {
        return;
      }

      // The form has unsaved changes — submit it instead of navigating.
      // Append a redirect_to parameter so the server sends the user to
      // the clicked step after saving.
      event.preventDefault();
      const action = new URL(
        form.getAttribute("action") || globalThis.location.href,
      );
      action.searchParams.set("redirect_to", link.pathname);
      form.action = action.toString();
      form.requestSubmit(submitBtn);
    });
  });
}
