// Enforce date format DD-MM-YYYY for date_of_birth inputs
window.addEventListener("load", () => {
  const dateInputs = document.querySelectorAll(
    'input[name="date_of_birth"]',
  ) as NodeListOf<HTMLInputElement>;
  dateInputs.forEach((input: HTMLInputElement) => {
    input.addEventListener("input", () => {
      const raw = input.value.replace(/[^\d-]/g, "");
      const digits = raw.replace(/\D/g, "").slice(0, 8);
      let hasFirstDash = raw.includes("-");
      let hasSecondDash =
        hasFirstDash && raw.indexOf("-", raw.indexOf("-") + 1) !== -1;

      let day = digits.slice(0, 2);
      let month = digits.slice(2, 4);
      let year = digits.slice(4);

      // Auto-insert leading zeros and dashes for the day
      if (
        day.length === 1 &&
        month.length === 0 &&
        (hasFirstDash || day > "3")
      ) {
        day = `0${day}`;
        hasFirstDash = true;
      }

      let formatted = day;

      if (day.length === 2 && (hasFirstDash || month.length > 0)) {
        formatted += "-";
      }

      // Auto-insert leading zeros and dashes for the month
      if (
        month.length === 1 &&
        year.length === 0 &&
        (hasSecondDash || month > "1")
      ) {
        month = `0${month}`;
        hasSecondDash = true;
      }

      formatted += month;

      if (month.length === 2 && (hasSecondDash || year.length > 0)) {
        formatted += "-";
      }

      // Auto-complete year for 2-digit inputs
      if (year.length > 0 && year[0] > "2") {
        year = `19${year}`;
      }

      formatted += year;
      input.value = formatted;
    });
  });
});
