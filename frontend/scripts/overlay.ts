const OVERLAY_STORAGE_KEY = "has-overlay";

const updateOverlayTransition = () => {
  const overlay: HTMLElement | null = document.querySelector(".overlay");
  const previousHadOverlay =
    globalThis.sessionStorage?.getItem(OVERLAY_STORAGE_KEY) === "true";

  if (overlay && previousHadOverlay) {
    document.body.classList.add("overlay-skip-anim");
  } else {
    document.body.classList.remove("overlay-skip-anim");
  }

  globalThis.sessionStorage?.setItem(
    OVERLAY_STORAGE_KEY,
    overlay ? "true" : "false",
  );
};

window.addEventListener("load", () => {
  updateOverlayTransition();

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

window.addEventListener("pagehide", () => {
  const overlay: HTMLElement | null = document.querySelector(".overlay");
  globalThis.sessionStorage?.setItem(
    OVERLAY_STORAGE_KEY,
    overlay ? "true" : "false",
  );
});
