export default function setupReadMore() {
    document.querySelectorAll(".read-more-button").forEach(readMoreElem => {
        const targetElement = document.querySelector(`#${readMoreElem.getAttribute("target")}`);
        const readLessElem = targetElement?.querySelector(".read-less-button");
        readMoreElem.addEventListener("click", _ => {
            targetElement?.classList.remove("hidden");
            readMoreElem.classList.add("hidden");
        });
        readLessElem?.addEventListener("click", _ => {
            targetElement?.classList.add("hidden");
            readMoreElem.classList.remove("hidden");
        })
    });
}
