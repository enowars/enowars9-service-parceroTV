document.addEventListener("DOMContentLoaded", function () {
    const params = new URLSearchParams(window.location.search);
    const filename = params.get("file");
    const video_id = params.get("id")

    async function getVideoInfo(path) {
        try {
            const res = await fetch("/get_video_info/" + path);
            const videoInfo = await res.json();

            if (videoInfo) {
                document.getElementById("name").innerText = videoInfo.name || "Untitled";
                document.getElementById("description").innerText = videoInfo.description || "";

                if (videoInfo.is_private == 1) {
                    const form = document.getElementById("commentForm");
                    if (form) form.remove();
                }
            } else {
                console.warn("videoInfo is null or undefined");
            }

            if (videoInfo.is_private) {
                document.getElementById("commentHeader").remove();
                const form = document.getElementById("commentForm");
                if (form) form.remove();
            }
        }
        catch {
            const header = document.getElementById("commentHeader").remove();
            const form = document.getElementById("commentForm");
            if (form) form.remove();
        }

    }

    getVideoInfo(filename);
    if (video_id) {
        document.getElementById("videoID").setAttribute('value', video_id);
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