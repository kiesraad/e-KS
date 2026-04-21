// Assist date-of-birth inputs with numeric formatting and dash handling.
const DATE_INPUT_ROW_SELECTOR = 'span[class="date-input-row"]';

const DATE_INPUT_SELECTOR = 'input[name="date_of_birth"]';

const DAY_INPUT_SELECTOR = 'input[name="day_of_birth"]';
const MONTH_INPUT_SELECTOR = 'input[name="month_of_birth"]';
const YEAR_INPUT_SELECTOR = 'input[name="year_of_birth"]';

const DASH_SELECTOR = ".dash";

const DAY_DIGITS = 2;
const MONTH_DIGITS = 2;
const YEAR_DIGITS = 4;

const IGNORE_TAB_DURATION_MS = 400;

type Inputs = {
  dayInput: HTMLInputElement;
  monthInput: HTMLInputElement;
  yearInput: HTMLInputElement;
  actual: HTMLInputElement;
};

type DateField = {
  inputs: Inputs;
  dashes: NodeListOf<HTMLElement>;
};

// Collect all date of birth inputs on the page.
function getDateInputs(): DateField[] {
  const rows = document.querySelectorAll(DATE_INPUT_ROW_SELECTOR);
  return [...rows].map((row) => ({
    inputs: {
      dayInput: row.querySelector(DAY_INPUT_SELECTOR) as HTMLInputElement,
      monthInput: row.querySelector(MONTH_INPUT_SELECTOR) as HTMLInputElement,
      yearInput: row.querySelector(YEAR_INPUT_SELECTOR) as HTMLInputElement,
      actual: row.querySelector(DATE_INPUT_SELECTOR) as HTMLInputElement,
    },
    dashes: row.querySelectorAll(DASH_SELECTOR),
  }));
}

function sanitize(input: HTMLInputElement) {
  input.value = input.value.replaceAll(/[^\d]/g, "");
}

function handleDayInput(
  dayInput: HTMLInputElement,
  month_input: HTMLInputElement,
): boolean {
  sanitize(dayInput);
  if (dayInput.value > "31") {
    dayInput.value = `0${dayInput.value}`;
  }
  dayInput.value = dayInput.value.slice(0, DAY_DIGITS);
  if (dayInput.value.length === DAY_DIGITS) {
    month_input.focus();
    return true;
  }
  return false;
}

function handleMonthInput(
  monthInput: HTMLInputElement,
  year_input: HTMLInputElement,
): boolean {
  sanitize(monthInput);
  if (monthInput.value > "12") {
    monthInput.value = `0${monthInput.value}`;
  }
  monthInput.value = monthInput.value.slice(0, MONTH_DIGITS);
  if (monthInput.value.length === MONTH_DIGITS) {
    year_input.focus();
    return true;
  }
  return false;
}

function handleYearInput(yearInput: HTMLInputElement) {
  sanitize(yearInput);
  if (yearInput.value.length > 0 && yearInput.value[0] > "2") {
    yearInput.value = `19${yearInput.value}`;
  }
  yearInput.value = yearInput.value.slice(0, YEAR_DIGITS);
}

function formatSingleDigit(input: HTMLInputElement) {
  if (input.value.length !== 1) {
    return;
  }
  input.value = `0${input.value}`;
}

function updateActual(input: Inputs) {
  input.actual.value = `${input.dayInput.value}-${input.monthInput.value}-${input.yearInput.value}`;
}

function updateVisible(field: DateField) {
  // swap inputs for JavaScript enjoyers
  field.inputs.actual.type = "hidden";
  field.inputs.dayInput.type = "text";
  field.inputs.monthInput.type = "text";
  field.inputs.yearInput.type = "text";

  field.dashes.forEach((dash) => {
    dash.classList.remove("hidden");
  });

  if (field.inputs.actual.value === "") {
    return;
  }

  const parts = field.inputs.actual.value.split("-");
  field.inputs.dayInput.value = parts[0];
  field.inputs.monthInput.value = parts[1];
  field.inputs.yearInput.value = parts[2];
}

function tabAdvanceCheck(
  event: KeyboardEvent,
  ignoreNextTab: boolean,
): boolean {
  if (event.key === "Tab" && ignoreNextTab) {
    event.preventDefault();
    return false;
  }
  return ignoreNextTab;
}

// Enforce date format DD-MM-YYYY for date_of_birth inputs.
export default function dateInput() {
  let ignoreNextTab = false;
  const dateField = getDateInputs();

  dateField.forEach((field) => {
    const dayInput = field.inputs.dayInput;
    const monthInput = field.inputs.monthInput;
    const yearInput = field.inputs.yearInput;

    updateVisible(field);
    dayInput.addEventListener("input", () => {
      ignoreNextTab = handleDayInput(dayInput, monthInput);
      updateActual(field.inputs);
      setTimeout(() => (ignoreNextTab = false), IGNORE_TAB_DURATION_MS);
    });
    monthInput.addEventListener("input", () => {
      ignoreNextTab = handleMonthInput(monthInput, yearInput);
      updateActual(field.inputs);
      setTimeout(() => (ignoreNextTab = false), IGNORE_TAB_DURATION_MS);
    });
    yearInput.addEventListener("input", () => {
      handleYearInput(yearInput);
      updateActual(field.inputs);
      setTimeout(() => (ignoreNextTab = false), IGNORE_TAB_DURATION_MS);
    });

    [monthInput, yearInput].forEach((input) => {
      input.addEventListener(
        "keydown",
        (e) => (ignoreNextTab = tabAdvanceCheck(e, ignoreNextTab)),
      );
    });

    [dayInput, monthInput].forEach((input) => {
      input.addEventListener("blur", (_) => formatSingleDigit(input));
    });
  });
}
