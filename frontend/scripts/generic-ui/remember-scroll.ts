// This script allows to remember the scroll position of the page when submitting a form, and scroll back to it on page load
export default function setupRememberScroll() {
  const candidateTable = document.getElementById("add-candidate-table");
  const form = candidateTable?.closest("form");

  if (!candidateTable || !form) {
    return;
  }

  // on page load, check if there is a stored scroll position for the current url and scroll to it
  const storedScrollY = localStorage.getItem(globalThis.location.href);
  if (storedScrollY) {
    window.scrollTo(0, parseInt(storedScrollY, 10));
  }

  // on form submit, store current scroll position in local storage, given the current url as key
  form.addEventListener("submit", () => {
    localStorage.setItem(globalThis.location.href, window.scrollY.toString());
  });
}
