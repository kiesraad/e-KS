/// Make table rows clickable
export const setupClickCard = () => {
  document.querySelectorAll(".card.card-clickable").forEach((card) => {
    card.addEventListener("click", () => {
      const link = card.querySelector("a,button");
      if (link) {
        (link as HTMLButtonElement | HTMLLinkElement).click();
      }
    });
  });
};

if (typeof globalThis !== "undefined") {
  globalThis.addEventListener("load", setupClickCard);
}
