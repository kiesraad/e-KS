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

type DateField = {
  inputs: Inputs;
  dashes: NodeListOf<HTMLElement>;
};

type Inputs = {
  day_input: HTMLInputElement;
  month_input: HTMLInputElement;
  year_input: HTMLInputElement;
  actual: HTMLInputElement;
};

// Collect all date of birth inputs on the page.
function getDateInputs(): DateField[] {
  const rows = document.querySelectorAll(DATE_INPUT_ROW_SELECTOR);
  return [...rows].map((row) => ({
    inputs: {
      day_input: row.querySelector(DAY_INPUT_SELECTOR) as HTMLInputElement,
      month_input: row.querySelector(MONTH_INPUT_SELECTOR) as HTMLInputElement,
      year_input: row.querySelector(YEAR_INPUT_SELECTOR) as HTMLInputElement,
      actual: row.querySelector(DATE_INPUT_SELECTOR) as HTMLInputElement,
    },
    dashes: row.querySelectorAll(DASH_SELECTOR),
  }));
}

function handleDayInput(
  dayInput: HTMLInputElement,
  month_input: HTMLInputElement,
) {
  if (dayInput.value > "3") {
    dayInput.value = `0${dayInput.value}`;
  }
  dayInput.value = dayInput.value.slice(0, DAY_DIGITS);
  if (dayInput.value.length === DAY_DIGITS) {
    month_input.focus();
  }
}

function handleMonthInput(
  monthInput: HTMLInputElement,
  year_input: HTMLInputElement,
) {
  if (monthInput.value > "12") {
    monthInput.value = `0${monthInput.value}`;
  }
  monthInput.value = monthInput.value.slice(0, MONTH_DIGITS);
  if (monthInput.value.length === MONTH_DIGITS) {
    year_input.focus();
  }
}

function handleYearInput(yearInput: HTMLInputElement) {
  if (yearInput.value.length > 0 && yearInput.value[0] > "2") {
    yearInput.value = `19${yearInput.value}`;
  }
  yearInput.value = yearInput.value.slice(0, YEAR_DIGITS);
}

function updateActual(input: Inputs) {
  input.actual.value = `${input.day_input.value}-${input.month_input.value}-${input.year_input.value}`;
}

function updateVisible(field: DateField) {
  // swap inputs for JavaScript enjoyers
  field.inputs.actual.type = "hidden";
  field.inputs.day_input.type = "text";
  field.inputs.month_input.type = "text";
  field.inputs.year_input.type = "text";

  field.dashes.forEach((dash) => {
    dash.classList.remove("hidden");
  });

  if (field.inputs.actual.value === "") {
    return;
  }

  const parts = field.inputs.actual.value.split("-");
  field.inputs.day_input.value = parts[0];
  field.inputs.month_input.value = parts[1];
  field.inputs.year_input.value = parts[2];
}
// Enforce date format DD-MM-YYYY for date_of_birth inputs.
export default function dateInput() {
  const dateField = getDateInputs();
  dateField.forEach((field) => {
    updateVisible(field);
    field.inputs.day_input.addEventListener("input", () => {
      handleDayInput(field.inputs.day_input, field.inputs.month_input);
      updateActual(field.inputs);
    });
    field.inputs.month_input.addEventListener("input", () => {
      handleMonthInput(field.inputs.month_input, field.inputs.year_input);
      updateActual(field.inputs);
    });
    field.inputs.year_input.addEventListener("input", () => {
      handleYearInput(field.inputs.year_input);
      updateActual(field.inputs);
    });
  });
}
