document.addEventListener("DOMContentLoaded", function () {
    async function fetchMyProfile() {
        try {
        const res = await fetch("/get_my_profile");
        const user = await res.json();
        
        document.getElementById("name").innerText = user.name;
        document.getElementById("about").innerText = user.about;

        }
        catch {
            console.log("Error fetching my profile");
        }
    }
    fetchMyProfile();

    async function get_my_videos() {
        try{
            const res = await fetch("/get_my_videos");
            const videos = await res.json();

            const private_container = document.getElementById("private_video_list");
            const public_container = document.getElementById("public_video_list");
            for (const video of videos) {
                const div = document.createElement("div");
                div.className = "video-card";

                const img = document.createElement("img");
                img.src = video.thumbnail_path;
                img.alt = video.name;
                img.style.width = "200px";

                const link = document.createElement("a");
                const title = document.createElement("h3");
                title.textContent = video.name;
                link.appendChild(title);
                link.setAttribute("href", "app/videos?file=" + video.path + "&id=" + video.id);
                const p = document.createElement("p");
                p.textContent = video.description;

                div.appendChild(img);
                div.appendChild(link);
                div.appendChild(p);

                if (!video.is_private) {
                    public_container.appendChild(div);
                }
                else{
                    private_container.appendChild(div);
                }
            }
        }
        catch (e){
            console.error("Error fetching videos "+ e);
        }
    }
    get_my_videos()
});