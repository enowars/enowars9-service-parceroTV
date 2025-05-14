document.addEventListener("DOMContentLoaded", function () {
    const params = new URLSearchParams(window.location.search);
    const filename = params.get("file");

    if (filename) {
        const source = document.getElementById("video-source");
        source.src = `/videos/${filename}`;
        const player = document.getElementById("video-player");
        player.load();
    } else {
        document.body.innerHTML += "<p>No video specified.</p>";
    }
});