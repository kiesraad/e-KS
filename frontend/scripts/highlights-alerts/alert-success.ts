// Remove success query params after showing the success alert.
export default function alertSuccess() {
  const url = new URL(globalThis.location.href);

  if (url.searchParams.get("success") === "true") {
    url.searchParams.delete("success");
  }

  globalThis.history.replaceState({}, "", url.toString());
}
