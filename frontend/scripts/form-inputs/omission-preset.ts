// Fill the omission description and help-text fields when a preset is clicked.
export default function omissionPreset() {
  const title = document.querySelector<HTMLInputElement>(
    "[data-omission-title]",
  );
  const description = document.querySelector<HTMLTextAreaElement>(
    "[data-omission-description]",
  );
  const helpText = document.querySelector<HTMLTextAreaElement>(
    "[data-omission-help-text]",
  );
  const recoverable = document.querySelector<HTMLInputElement>(
    "[data-omission-recoverable]",
  );

  if (!description) {
    return;
  }

  document
    .querySelectorAll<HTMLButtonElement>("[data-omission-preset]")
    .forEach((button) => {
      button.addEventListener("click", () => {
        if (title) {
          title.value = button.dataset.title ?? "";
        }
        description.value = button.dataset.description ?? "";
        if (helpText) {
          helpText.value = button.dataset.helpText ?? "";
        }
        if (recoverable) {
          recoverable.checked = button.dataset.recoverable !== "false";
        }
        description.focus();
      });
    });
}
