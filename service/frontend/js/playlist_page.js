document.addEventListener("DOMContentLoaded", function () {

    async function fetchPlaylistData(playlistId) {
        const res = await fetch(`/get_videos_in_playlist/${encodeURIComponent(playlistId)}`);
        const playlist = await res.json();
        console.log("Fetched Playlist Data:", playlist);
        try {

            const container = document.getElementById("video-list");

            for (const video of playlist) {
                console.log("Adding video to playlist:", video);
                const div = document.createElement("div");
                div.className = "video-card";

                const img = document.createElement("img");
                img.src = video.thumbnail_path;
                img.alt = video.name;
                img.style.width = "200px";

                const link = document.createElement("a");
                const h3 = document.createElement("h3");
                h3.textContent = video.name;
                link.appendChild(h3);
                link.setAttribute("href", "app/videos?file=" + video.path + "&id=" + video.id);

                const creator_link = document.createElement("a");


                const creator = document.createElement("h4");
                const creatorName = await fetchUserName(video.userId);
                creator_link.setAttribute("href", "app/users?name=" + creatorName);
                creator.textContent = "By " + creatorName;

                creator_link.appendChild(creator);

                const p = document.createElement("p");
                p.textContent = video.description;

                const hr = document.createElement("hr");
                const div_info = document.createElement("div");
                div.appendChild(img);
                div_info.appendChild(link);
                div_info.appendChild(p);
                div_info.appendChild(creator_link);
                div.appendChild(div_info)
                container.appendChild(div);
            }
        }
        catch {
            console.log("Error Fetching Videos");
        }
    }

    async function fetchUserName(id) {
        try {
            const res = await fetch("/get_user_info/" + id);
            const user = await res.json();

            return user.name;
        } catch {
            console.log("Error Fetching User Name");
        }
    }

    fetchPlaylistData(new URLSearchParams(window.location.search).get("id"));

    fetch("/header")
        .then(res => res.text())
        .then(html => document.getElementById("header").innerHTML = html);

    fetch("/footer")
        .then(res => res.text())
        .then(html => document.getElementById("footer").innerHTML = html)

});