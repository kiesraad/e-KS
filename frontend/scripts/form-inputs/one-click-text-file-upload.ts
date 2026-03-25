export default function one_click_upload() {
  document.querySelectorAll(".one-click-upload").forEach((button) => {
    let target_id = button.getAttribute("target");
    if (!target_id) {
      return;
    }

    let target = document.getElementById(target_id) as HTMLInputElement;

    target.addEventListener("change", async () => {
      let file = target.files?.[0];

      if (!file) {
        return;
      }

      let file_content = await file?.text();

      const response = await fetch(globalThis.location.href, {
        method: "POST",
        body: file_content,
        headers: {
            "Content-Type": file.type,
        },
      });
      
      const html = await response.text();
      document.documentElement.innerHTML = html;
    });

    button.addEventListener("click", () => {
      target.click();
    });
  });
}
