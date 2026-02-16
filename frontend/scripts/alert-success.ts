if (typeof globalThis !== "undefined") {
  globalThis.addEventListener("load", () => {
    const url = new URL(globalThis.location.href);

    if (url.searchParams.get("success") === "true") {
      url.searchParams.delete("success");
    }

    globalThis.history.replaceState({}, "", url.toString());
  });
}
