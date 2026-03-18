export default function one_click_upload() {
    document.querySelectorAll(".one-click-upload").forEach((button) => {
        let target_id = button.getAttribute("target");
        if (!target_id) {
            return;
        }
        
        let target = document.getElementById(target_id);
        if (!target) {
            return;
        }

        target.addEventListener("change", () => {
            target.closest("form")?.submit();
        });

        button.addEventListener("click", () => {
            target.click();
        });
    });
}
