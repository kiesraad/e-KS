// This script allows to remember the scroll position of the page when submitting a form, and scroll back to it on page load
export default function setupRememberScroll() {
  const candidateTable = document.getElementById("add-candidate-table");
  const form = candidateTable?.closest("form");
  const search = document.getElementById("search");

  if (!candidateTable || !form || !search) {
    return;
  }

  // on page load, check if there is a stored scroll position for the current url and scroll to it
  const storedScrollY = localStorage.getItem(globalThis.location.pathname);
  if (storedScrollY) {
    window.scrollTo(0, Number.parseInt(storedScrollY, 10));
    localStorage.removeItem(globalThis.location.pathname);
  } else {
    search.focus({
      preventScroll: true,
    });
  }

  // on form submit, store current scroll position in local storage, given the current url as key
  form.addEventListener("submit", () => {
    localStorage.setItem(
      globalThis.location.pathname,
      window.scrollY.toString(),
    );
  });
}
