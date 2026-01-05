window.addEventListener("load", () => {
  const input = document.getElementById("locality") as HTMLInputElement | null;
  const datalist = document.getElementById("locality-suggestions");

  if (!input || !datalist) {
    return;
  }

  input.addEventListener("input", async () => {
    const q = input.value;

    if (q.length < 3) {
      return;
    }

    const res = await fetch(`/suggest?wp=${encodeURIComponent(q)}`);
    const suggestions: Array<string> = await res.json();

    datalist.innerHTML = "";
    suggestions.forEach((item) => {
      const option = document.createElement("option");
      option.value = item;
      datalist.appendChild(option);
    });
  });
});
