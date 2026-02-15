const url = new URL(globalThis.location.href);
const successParam = url.searchParams.get("success");

if (successParam === "true") {
  url.searchParams.delete("success");
  globalThis.history.replaceState({}, "", url.toString());
}
