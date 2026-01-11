window.addEventListener("load", () => {
  const overlay: HTMLElement | null = document.querySelector(".overlay");

  if (!overlay) {
    return;
  }

  document.addEventListener("keyup", (event) => {
    // check that we are not in an input field
    const activeElement = document.activeElement;
    if ((activeElement as HTMLElement).isContentEditable) {
      return;
    }

    if (event.key === "Escape") {
      const close: HTMLAnchorElement | null =
        document.querySelector("a.close-overlay");

      if (close) {
        close.click();
      }
    }
  });
});
