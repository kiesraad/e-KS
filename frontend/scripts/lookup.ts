window.addEventListener("load", () => {
  const postalCodeInput = document.getElementById(
    "postal_code",
  ) as HTMLInputElement | null;
  const houdeNumberInput = document.getElementById(
    "house_number",
  ) as HTMLInputElement | null;
  const localityInput = document.getElementById(
    "locality",
  ) as HTMLInputElement | null;
  const streetNameInput = document.getElementById(
    "street_name",
  ) as HTMLInputElement | null;

  // only run if all form fields are found
  if (
    !postalCodeInput ||
    !houdeNumberInput ||
    !localityInput ||
    !streetNameInput
  ) {
    return;
  }

  const lookup = async () => {
    // only perform lookup when postal code and house number are filled and locality and street name are empty
    if (
      !postalCodeInput.value ||
      !houdeNumberInput.value ||
      localityInput.value ||
      streetNameInput.value
    ) {
      return;
    }

    // fetch address data from backend
    const url = `/lookup?pc=${postalCodeInput.value}&n=${houdeNumberInput.value}`;
    const response = await fetch(url, {
      method: "GET",
      headers: {
        Accept: "application/json",
      },
    });

    if (response.ok) {
      const data = await response.json();
      if (data.wp) {
        localityInput.value = data.wp;
      }
      if (data.pr) {
        streetNameInput.value = data.pr;
      }
    }
  };

  postalCodeInput.addEventListener("change", lookup);
  houdeNumberInput.addEventListener("change", lookup);
});
