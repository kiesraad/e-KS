window.addEventListener("load", () => {
  const modal: HTMLDialogElement | null =
    document.querySelector("dialog.modal");

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
