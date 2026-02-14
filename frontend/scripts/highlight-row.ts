const highlightParam = () => {
  const params = new URLSearchParams(globalThis.location.search);
  return params.get("highlight");
};

const highlightRow = () => {
  const personId = highlightParam();

  if (!personId) {
    return;
  }

  const row = document.querySelector(`tr[data-id="${personId}"]`);

  if (!row) {
    return;
  }

  row.classList.add("highlighted");
  row.scrollIntoView({ behavior: "auto", block: "center" });

  setTimeout(() => {
    row.classList.remove("highlighted");
  }, 2000);
};

if (typeof globalThis !== "undefined") {
  globalThis.addEventListener("load", highlightRow);
}
