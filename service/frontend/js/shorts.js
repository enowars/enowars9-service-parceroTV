document.addEventListener("DOMContentLoaded", function () {
    const uploadPanel = document.getElementById("panel");
    function openPanel() {
        uploadPanel.classList.toggle("hidden");
    }

    document.getElementById("uploadShortBtn").addEventListener("click", openPanel);

    fetch("/header")
        .then(res => res.text())
        .then(html => document.getElementById("header").innerHTML = html);

    fetch("/footer")
        .then(res => res.text())
        .then(html => document.getElementById("footer").innerHTML = html)

});