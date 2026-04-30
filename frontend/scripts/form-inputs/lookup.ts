// Auto-fill address fields using postal code and house number lookups.
export default function addressLookup() {
  const postalCodeInput = document.getElementById(
    "postal_code",
  ) as HTMLInputElement | null;
  const houseNumberInput = document.getElementById(
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
    !houseNumberInput ||
    !localityInput ||
    !streetNameInput
  ) {
    return;
  }

  const lookup = async () => {
    resetSuccessHighlight();

    postalCodeInput.value = postalCodeInput.value
      .replaceAll(/\s/g, "")
      .toUpperCase();

    // only perform lookup when postal code and house number are filled
    if (!postalCodeInput.value || !houseNumberInput.value) {
      return;
    }

    // fetch address data from backend
    const url = `/lookup?pc=${postalCodeInput.value}&n=${houseNumberInput.value}`;
    const response = await fetch(url, {
      method: "GET",
      headers: {
        Accept: "application/json",
      },
    });

    if (response.ok) {
      const data = await response.json();
      // fill in locality and street name if data is available
      if (data.wp && data.pr) {
        localityInput.value = data.wp;
        streetNameInput.value = data.pr;

        // highlight the address fields to indicate they were auto-filled
        postalCodeInput.closest(".form-field")?.classList.add("success");
        houseNumberInput.closest(".form-field")?.classList.add("success");
        localityInput.closest(".form-field")?.classList.add("success");
        streetNameInput.closest(".form-field")?.classList.add("success");
      }
    }
  };

  function resetSuccessHighlight() {
    postalCodeInput?.closest(".form-field")?.classList.remove("success");
    houseNumberInput?.closest(".form-field")?.classList.remove("success");
    localityInput?.closest(".form-field")?.classList.remove("success");
    streetNameInput?.closest(".form-field")?.classList.remove("success");
  }

  postalCodeInput.addEventListener("change", lookup);
  houseNumberInput.addEventListener("change", lookup);
  localityInput.addEventListener("change", resetSuccessHighlight);
  streetNameInput.addEventListener("change", resetSuccessHighlight);
}
