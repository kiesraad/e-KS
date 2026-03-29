const FILE_INPUT_SELECTOR = "#import_file";
const FILE_FIELD_SELECTOR = "[data-import-file-field]";
const FILE_TRIGGER_SELECTOR = "[data-import-file-trigger]";

type ImportElements = {
  form: HTMLFormElement;
  fileInput: HTMLInputElement;
  fileField: HTMLElement | null;
  triggers: HTMLButtonElement[];
};

function getImportElements(): ImportElements | null {
  const fileInput =
    document.querySelector<HTMLInputElement>(FILE_INPUT_SELECTOR);
  const form = fileInput?.closest("form");
  const triggers = Array.from(
    document.querySelectorAll<HTMLButtonElement>(FILE_TRIGGER_SELECTOR),
  );

  if (!fileInput || !form || triggers.length === 0) {
    return null;
  }

  return {
    form,
    fileInput,
    fileField: document.querySelector<HTMLElement>(FILE_FIELD_SELECTOR),
    triggers,
  };
}

function hideFileField(fileField: HTMLElement | null) {
  fileField?.classList.add("hidden");
}

function openFilePicker(fileInput: HTMLInputElement) {
  fileInput.value = "";
  fileInput.click();
}

function submitImportForm(
  form: HTMLFormElement,
  submitter: HTMLButtonElement | null,
) {
  if (submitter) {
    form.requestSubmit(submitter);
    return;
  }

  form.requestSubmit();
}

export default function setupFileImport() {
  const elements = getImportElements();

  if (!elements) {
    return;
  }

  const { form, fileInput, fileField, triggers } = elements;
  let submitter: HTMLButtonElement | null = null;

  hideFileField(fileField);

  triggers.forEach((trigger) => {
    trigger.addEventListener("click", (event) => {
      event.preventDefault();
      submitter = trigger;
      openFilePicker(fileInput);
    });
  });

  fileInput.addEventListener("change", () => {
    if (!fileInput.files || fileInput.files.length === 0) {
      return;
    }

    submitImportForm(form, submitter);
  });
}
