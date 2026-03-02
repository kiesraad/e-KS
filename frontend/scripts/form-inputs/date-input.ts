// Assist date-of-birth inputs with numeric formatting and dash handling.
const DATE_INPUT_SELECTOR = 'input[name="date_of_birth"]';
const MAX_DATE_DIGITS = 8;

type DateParts = {
  day: string;
  month: string;
  year: string;
};

type DashState = {
  hasFirstDash: boolean;
  hasSecondDash: boolean;
};

type ParsedInput = {
  digits: string;
  dashState: DashState;
};

// Collect all date of birth inputs on the page.
function getDateInputs(): NodeListOf<HTMLInputElement> {
  return document.querySelectorAll(DATE_INPUT_SELECTOR);
}

// Strip unsupported characters and track whether user typed dashes.
function parseInput(value: string): ParsedInput {
  const raw = value.replaceAll(/[^\d-]/g, "");
  const digits = raw.replaceAll(/\D/g, "").slice(0, MAX_DATE_DIGITS);
  const hasFirstDash = raw.includes("-");
  const hasSecondDash = hasFirstDash && raw.includes("-", raw.indexOf("-") + 1);

  return {
    digits,
    dashState: { hasFirstDash, hasSecondDash },
  };
}

// Split the digit stream into day, month, and year parts.
function splitDigits(digits: string): DateParts {
  return {
    day: digits.slice(0, 2),
    month: digits.slice(2, 4),
    year: digits.slice(4),
  };
}

// Auto-pad day and advance to month when day is complete or dashed.
function normalizeDay(
  parts: DateParts,
  dashState: DashState,
): { parts: DateParts; dashState: DashState } {
  const { day, month, year } = parts;
  if (
    day.length === 1 &&
    month.length === 0 &&
    (dashState.hasFirstDash || day > "3")
  ) {
    return {
      parts: { day: `0${day}`, month, year },
      dashState: { ...dashState, hasFirstDash: true },
    };
  }

  return { parts, dashState };
}

// Auto-pad month and advance to year when month is complete or dashed.
function normalizeMonth(
  parts: DateParts,
  dashState: DashState,
): { parts: DateParts; dashState: DashState } {
  const { day, month, year } = parts;
  if (
    month.length === 1 &&
    year.length === 0 &&
    (dashState.hasSecondDash || month > "1")
  ) {
    return {
      parts: { day, month: `0${month}`, year },
      dashState: { ...dashState, hasSecondDash: true },
    };
  }

  return { parts, dashState };
}

// Expand a year starting with 3-9 to 19xx.
function normalizeYear(year: string): string {
  if (year.length > 0 && year[0] > "2") {
    return `19${year}`;
  }

  return year.slice(0, 4);
}

// Reassemble parts into a DD-MM-YYYY formatted string.
function buildFormattedDate(parts: DateParts, dashState: DashState): string {
  const { day, month, year } = parts;
  let formatted = day;

  if (day.length === 2 && (dashState.hasFirstDash || month.length > 0)) {
    formatted += "-";
  }

  formatted += month;

  if (month.length === 2 && (dashState.hasSecondDash || year.length > 0)) {
    formatted += "-";
  }

  return formatted + year;
}

// Normalize user input into a consistent DD-MM-YYYY shape.
export function formatDateValue(value: string): string {
  const { digits, dashState } = parseInput(value);
  let parts = splitDigits(digits);
  let nextDashState = dashState;

  ({ parts, dashState: nextDashState } = normalizeDay(parts, nextDashState));
  ({ parts, dashState: nextDashState } = normalizeMonth(parts, nextDashState));
  parts = { ...parts, year: normalizeYear(parts.year) };

  return buildFormattedDate(parts, nextDashState);
}

// Enforce date format DD-MM-YYYY for date_of_birth inputs.
export default function dateInput() {
  const dateInputs = getDateInputs();
  dateInputs.forEach((input) => {
    input.addEventListener("input", () => {
      input.value = formatDateValue(input.value);
    });
  });
}
