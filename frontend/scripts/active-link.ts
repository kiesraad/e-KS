window.addEventListener("load", () => {
  document.querySelectorAll("a").forEach((link) => {
    if (link.href === globalThis.location.href) {
      link.classList.add("active");
    }
  });
});
