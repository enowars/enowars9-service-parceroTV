document.addEventListener("DOMContentLoaded", function () {
    const params = new URLSearchParams(window.location.search);
    const filename = params.get("file");
    const video_id = params.get("id");
    console.log("hi");
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

    getComments(video_id);
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
    console.log("test");
    async function getComments(video_id) {
        console.log("sad");
        try {
            const res = await fetch("/get_comments/" + video_id);
            const comments = await res.json();


            const container = document.getElementById("commentList");
            for (const comment of comments) {
                const commentDiv = document.createElement("div");
                commentDiv.className = "comment";

                const userLink = document.createElement("a");
                userLink.href = `/app/users?id=${comment.user_id}`;
                userLink.textContent = comment.username;
                userLink.className = "comment-user";


                const timestamp = document.createElement("span");
                timestamp.textContent = ` • ${comment.created_at}`;
                timestamp.className = "comment-time";


                const header = document.createElement("div");
                header.className = "comment-header";
                header.appendChild(userLink);
                header.appendChild(timestamp);

                const body = document.createElement("p");
                body.textContent = comment.comment;
                body.className = "comment-body";

                commentDiv.appendChild(header);
                commentDiv.appendChild(body);
                container.appendChild(commentDiv);
            }
        }
        catch (e) {
            console.log("error fetching comments" + e);
        }

    }
    fetch("/header")
        .then(res => res.text())
        .then(html => document.getElementById("header").innerHTML = html);

    fetch("/footer")
        .then(res => res.text())
        .then(html => document.getElementById("footer").innerHTML = html)

});
