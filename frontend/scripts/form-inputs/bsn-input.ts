// Toggle BSN input availability when "no BSN" is confirmed.
export default function bsnInput() {
  const bsnInput = document.getElementById("bsn") as HTMLInputElement | null;
  const noBsnConfirmedInput = document.getElementById(
    "no_bsn_confirmed",
  ) as HTMLInputElement | null;

  if (bsnInput && noBsnConfirmedInput) {
    let tmpValue = bsnInput.value;

    const toggleBsnInput = () => {
      if (noBsnConfirmedInput.checked) {
        tmpValue = bsnInput.value;
        bsnInput.value = "";
        bsnInput.disabled = true;
        bsnInput.classList.add("disabled");
      } else {
        bsnInput.value = tmpValue;
        bsnInput.disabled = false;
        bsnInput.classList.remove("disabled");
      }
    };

    noBsnConfirmedInput.addEventListener("change", toggleBsnInput);
    toggleBsnInput();
  }
}
