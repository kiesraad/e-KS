window.addEventListener("load", () => {
  document.querySelectorAll("a").forEach((link) => {
    if (link.pathname === globalThis.location.pathname) {
      link.classList.add("active");
    }
  });
});
