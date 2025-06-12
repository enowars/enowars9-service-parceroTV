document.addEventListener("DOMContentLoaded", function () {
    const params = new URLSearchParams(window.location.search);
    let id;
    const name = params.get("name");

    async function fetchUser(name) {
        try {
            const res = await fetch("/get_user_info_with_name/" + name);
            const user = await res.json();

            document.getElementById("name").innerText = user.name;
            document.getElementById("about").innerText = user.about;
            id = user.id;
            console.log(user.id)
        }
        catch {
            console.log("Fetching User Data not possible")
        }
    }

    async function fetchUserName(id) {
        try {
            const res = await fetch("/get_user_info/" + id);
            const user = await res.json();

            return user.name;
        }
        catch {
            console.log("Fetching User Data not possible")
        }
    }
    async function init() {
        await fetchUser(name);
        if (!id) {
            console.error("User ID is missing. Aborting.");
            return;
        }
        await loadPublicVideos();
        await loadPrivateVideoInfo(id);
    }

    async function loadPublicVideos() {
        try {
            const res = await fetch("/get_videos/" + id);
            const videos = await res.json();

            const container = document.getElementById("public_video_list");

            for (const video of videos) {
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

                creator_link.setAttribute("href", "app/users?id=" + video.userId);
                const creator = document.createElement("h4");
                const creatorName = await fetchUserName(video.userId);
                creator.textContent = "By " + creatorName;

                creator_link.appendChild(creator);

                const p = document.createElement("p");
                p.textContent = video.description;

                const hr = document.createElement("hr");

                div.appendChild(img);
                div.appendChild(link);
                div.appendChild(p);
                div.appendChild(hr);

                container.appendChild(div);
            }
        }
        catch (e) {
            console.log("Error Fetching Videos" + e);
        }
    }

    async function loadPrivateVideoInfo(id) {
        try {
            const res = await fetch("/get_private_videos/" + id);
            const videos = await res.json();

            const container = document.getElementById("private_video_list");

            for (const video of videos) {
                const div = document.createElement("div");
                div.className = "video-card";

                const img = document.createElement("img");
                img.src = video.thumbnail_path;
                img.alt = video.name;
                img.style.width = "200px";

                const h3 = document.createElement("h3");
                h3.className = "video-name";
                h3.textContent = video.name;

                const creator_link = document.createElement("a");

                creator_link.setAttribute("href", "app/users?id=" + video.userId);
                const creator = document.createElement("h4");
                const creatorName = await fetchUserName(video.userId);
                creator.textContent = "By " + creatorName;

                creator_link.appendChild(creator);

                const p = document.createElement("p");
                p.className = "video-location"
                p.textContent = "Genre:\n \n" + video.location;

                const hr = document.createElement("hr");

                div.appendChild(img);
                div.appendChild(h3);
                div.appendChild(p);
                div.appendChild(hr);

                container.appendChild(div);
            }
        }
        catch (e) {
            console.log("Error Fetching Videos" + e);
        }
    }
    init();
    fetch("/header")
        .then(res => res.text())
        .then(html => document.getElementById("header").innerHTML = html);

    fetch("/footer")
        .then(res => res.text())
        .then(html => document.getElementById("footer").innerHTML = html);

});