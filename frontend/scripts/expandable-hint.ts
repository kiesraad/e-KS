window.addEventListener("load", () => {
    document.querySelectorAll(".hint-expander").forEach((element) => {
        element.addEventListener("click", (event) => {
            let expandable = element.querySelector<HTMLElement>(".hint-expansion");
            if (expandable?.style.display != "hidden") {
                expandable
            }
        })
    });
});
