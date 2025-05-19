document.addEventListener("DOMContentLoaded", function () {
    const params = new URLSearchParams(window.location.search);
    const filename = params.get("file");
    const video_id = params.get("id")

    async function getVideoInfo(path) {
        const res = await fetch("/get_video_info/"+path);
        const videoInfo = await res.json();

        document.getElementById("name").innerText = videoInfo.name || "Untitled";
        document.getElementById("description").innerText = videoInfo.description || "";
    }

    getVideoInfo(filename);
    if (filename) {
        const source = document.getElementById("video-source");
        source.src = `${filename}`;
        const player = document.getElementById("video-player");
        player.load();
    } else {
        document.body.innerHTML += "<p>No video specified.</p>";
    }
});