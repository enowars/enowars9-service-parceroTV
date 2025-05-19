document.addEventListener("DOMContentLoaded", function () {
    const params = new URLSearchParams(window.location.search);
    const filename = params.get("file");
    const video_id = params.get("id")

    if (video_id) {
        const idfield = document.getElementsByName("video_id")[0];
        idfield.value = video_id;
    }
    if (filename) {
        const source = document.getElementById("video-source");
        source.src = `${filename}`;
        const player = document.getElementById("video-player");
        player.load();
    } else {
        document.body.innerHTML += "<p>No video specified.</p>";
    }
});