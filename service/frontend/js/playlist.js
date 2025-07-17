document.addEventListener("DOMContentLoaded", function () {

    const uploadPanel = document.getElementById("panel");
    function openPanel() {
        uploadPanel.classList.toggle("hidden");
    }

    document.getElementById("createPlaylistBtn").addEventListener("click", openPanel);

    async function fetchUserNames() {
        try {
            const res = await fetch("/get_all_users");
            const users_json = await res.json();
            for (const user of users_json) {
                const users = document.getElementById("users");
                const option = document.createElement("option");
                option.value = user.id;
                option.textContent = user.name;
                users.appendChild(option);
            }
        }
        catch {
            console.log("Fetching User Data not possible");
        }
    }

    async function fetchVideos() {
        try {
            const res = await fetch("/api/fetch_all_videos");
            const videos = await res.json();

            const videoSelect = document.getElementById("videos");
            for (const video of videos) {
                const option = document.createElement("option");
                option.value = parseInt(video.id);
                option.textContent = video.name;
                videoSelect.appendChild(option);
            }
        }
        catch {
            console.log("Fetching Video Data not possible");
        }
    }

    async function fetchPublicPlaylists() {
        try {
            const res = await fetch("/get_playlists_public");
            const playlists = await res.json();
            populatePlaylists(playlists);
        }
        catch {
            console.log("Fetching Public Playlist Data not possible");
        }
    }


    function populatePlaylists(playlists, isPrivate = false) {
        const playlistContainer = isPrivate ? document.getElementById("private_playlists") : document.getElementById("public_playlists");
        for (const playlist of playlists) {
            const div = document.createElement("div");
            div.className = "playlist";
            const title = document.createElement("h2");
            title.className = "playlist-title";
            title.textContent = playlist.name ? playlist.name : "Unnamed Playlist";

            const description = document.createElement("div");
            description.className = "playlist-description";
            description.textContent = playlist.description ? playlist.description : "No description available.";

            const link = document.createElement("a");
            link.href = `/app/playlist_page?id=${encodeURIComponent(playlist.id)}`;
            link.className = "view-playlist-link";

            const button = document.createElement("button");
            button.className = "view-playlist-button";
            button.textContent = "View Playlist";

            link.appendChild(button);
            div.appendChild(title);
            div.appendChild(description);
            div.appendChild(link);
            playlistContainer.appendChild(div);
        }
    }

    async function fetchPrivatePlaylists() {
        try {
            const res = await fetch("/get_playlists_private");
            const playlists = await res.json();
            populatePlaylists(playlists, true);
        }
        catch {
            console.log("Fetching Private Playlist Data not possible");
        }
    }

    fetchUserNames();
    fetchVideos();
    fetchPublicPlaylists();
    fetchPrivatePlaylists();


    fetch("/header")
        .then(res => res.text())
        .then(html => document.getElementById("header").innerHTML = html);

    fetch("/footer")
        .then(res => res.text())
        .then(html => document.getElementById("footer").innerHTML = html)

});