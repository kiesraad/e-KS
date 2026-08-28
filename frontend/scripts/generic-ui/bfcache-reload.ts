// Pages render per-session data, so a page restored from the browser's
// back/forward cache may be stale (Cache-Control: no-store does not keep
// pages out of that cache). Refetch instead of showing the restored copy.
export default function setupBfcacheReload() {
  window.addEventListener("pageshow", (event) => {
    if (event.persisted) {
      globalThis.location.reload();
    }
  });
}
