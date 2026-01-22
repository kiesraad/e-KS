/// Make table rows clickable
window.addEventListener("load", () => {
  document.querySelectorAll(".card.card-clickable").forEach((card) => {
    card.addEventListener("click", () => {
      const link = card.querySelector("a,button");
      if (link) {
        (link as HTMLButtonElement | HTMLLinkElement).click();
      }
    });
  });
});
