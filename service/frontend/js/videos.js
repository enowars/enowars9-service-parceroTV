document.addEventListener("DOMContentLoaded", function () {
    const params = new URLSearchParams(window.location.search);
    const like_btn = document.getElementById("likeBtn");
    const dislike_btn = document.getElementById("dislikeBtn");
    const filename = params.get("file");
    const video_id = params.get("id");
    async function getVideoInfo(path) {
        try {
            const res = await fetch("/get_video_info/" + path);
            const videoInfo = await res.json();

            if (videoInfo) {
                document.getElementById("name").innerText = videoInfo.name || "Untitled";
                document.getElementById("description").innerText = videoInfo.description || "";
                document.getElementById("likeCount").innerText = videoInfo.likes || 0;
                document.getElementById("dislikeCount").innerText = videoInfo.dislikes || 0;
                document.getElementById("viewCount").innerText = videoInfo.clicks || 0;

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

    async function getLikeStatus() {
        try {
            const res = await fetch("/get_like_status/" + video_id);
            const likeStatus = await res.json();
            if (likeStatus.status === "liked") {
                like_btn.classList.toggle("liked");
                
            }
            if (likeStatus.status === "disliked") {
                dislike_btn.classList.toggle("liked");
            }
        } catch {

        }
    }

    getLikeStatus();

    async function update_like() {
    try {
        const alreadyLiked = like_btn.classList.contains("liked");
        const alreadyDisliked = dislike_btn.classList.contains("liked");

        if (alreadyLiked) {
            return;
        }

        
        await fetch("/update_like/" + video_id, {
            method: "POST"
        });

        like_btn.classList.add("liked");
        dislike_btn.classList.remove("liked");

        
        const likeCountEl = document.getElementById("likeCount");
        const dislikeCountEl = document.getElementById("dislikeCount");

        likeCountEl.textContent = parseInt(likeCountEl.textContent) + 1;

        if (alreadyDisliked) {
            dislikeCountEl.textContent = parseInt(dislikeCountEl.textContent) - 1;
        }

    } catch (err) {
        console.error("Like update failed:", err);
    }
}


    async function update_dislike() {
    try {
        const alreadyDisliked = dislike_btn.classList.contains("liked");
        const alreadyLiked = like_btn.classList.contains("liked");

        if (alreadyDisliked) {
    
            return;
        }

        await fetch("/update_dislike/" + video_id, {
            method: "POST"
        });

        
        dislike_btn.classList.add("liked");
        like_btn.classList.remove("liked");

        const likeCountEl = document.getElementById("likeCount");
        const dislikeCountEl = document.getElementById("dislikeCount");

        if (alreadyLiked) {
            
            likeCountEl.textContent = parseInt(likeCountEl.textContent) - 1;
        }

        dislikeCountEl.textContent = parseInt(dislikeCountEl.textContent) + 1;

    } catch (err) {
        console.error("Dislike update failed:", err);
    }
}


    async function increase_view_count() {
        try {
            fetch("/increase_view_count/" + video_id, {
                method: "POST"
            });
        } catch {

        }
    }
    like_btn.addEventListener("click", update_like);
    dislike_btn.addEventListener("click", update_dislike);
    increase_view_count();
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

    async function getComments(video_id) {

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
