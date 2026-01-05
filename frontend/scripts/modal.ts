window.addEventListener("load", () => {
  const modal = document.querySelector(
    "dialog.modal",
  ) as HTMLDialogElement | null;

  if (modal) {
    const openButton = document.querySelector("button.open-modal");
    const closeButton = modal.querySelector("button.close-modal");

    if (closeButton) {
      closeButton.addEventListener("click", (e) => {
        e.preventDefault();
        modal.close();
      });
    }

    if (openButton) {
      openButton.addEventListener("click", (e) => {
        e.preventDefault();
        modal.showModal();
      });
    }
  }
});
