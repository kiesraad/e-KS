/// Make cards with data-href clickable, except when clicking on a link inside the card
export default function setupClickCard() {
  document.querySelectorAll<HTMLElement>(".card[data-href]").forEach((card) => {
    card.addEventListener("click", (event) => {
      if (event.target instanceof HTMLElement && event.target.closest("a")) {
        return;
      }
      const href = card.dataset.href;
      if (href) {
        globalThis.location.href = href;
      }
    });
  });
}
