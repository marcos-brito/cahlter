function changeColorscheme(colorscheme) {
    let html = document.querySelector("html");

    localStorage.setItem("colorscheme", colorscheme);
    html.className = colorscheme;
}

function setupThemeButton() {
    let button = document.querySelector(".theme-button");
    let popup = document.querySelector(".theme-popup");
    let items = document.querySelectorAll(".theme-popup__item");

    function handleButtonClick() {
        let { left, top } = button.getBoundingClientRect();

        popup.style.left = `${left - 100}px`;
        popup.style.top = `${top + 25}px`;

        if (popup.style.display == "none") {
            popup.style.display = "block";
            return;
        }

        popup.style.display = "none";
    }

    function handleDocumentClick(event) {
        if (!popup.contains(event.target) && event.target != button) {
            popup.style.display = "none";
        }
    }

    for (let item of items) {
        item.addEventListener("click", () => changeColorscheme(item.textContent));
    }

    button.addEventListener("click", handleButtonClick);
    document.addEventListener("click", handleDocumentClick);
    window.addEventListener("resize", () => (popup.style.display = "none"));
}

if (localStorage.getItem("colorscheme")) {
    changeColorscheme(localStorage.getItem("colorscheme"));
}

setupThemeButton();
