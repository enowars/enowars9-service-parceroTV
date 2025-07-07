document.addEventListener("DOMContentLoaded", function () {
    const uploadPanel = document.getElementById("panel");
    function openPanel() {
        uploadPanel.classList.toggle("hidden");
    }

    document.getElementById("uploadShortBtn").addEventListener("click", openPanel);


    document.getElementById('file').addEventListener('change', function (event) {
        const file = event.target.files[0];
        if (!file) return;



        const video = document.createElement("video")
        const url = URL.createObjectURL(file);
        video.src = url;

        video.onloadedmetadata = function () {
            const duration = video.duration;
            document.getElementById('duration').value = duration.toFixed(2);
            console.log("duration:", document.getElementById('duration').value)
            URL.revokeObjectURL(url);
        };
    });


    let allShorts = [];
    let currentIndex = 0;
    let loading = false;

    async function fetchAllShorts() {
        const res = await fetch("/get_shorts");
        allShorts = await res.json();
        if (allShorts.length === 0) return;
        appendShorts(3); 
    }

    function appendShorts(count) {
        const container = document.getElementById("shorts-container");

        for (let i = 0; i < count; i++) {
            const short = allShorts[currentIndex];

            const div = document.createElement("div");
            div.className = "short";
            div.innerHTML = `
            <div class="short-info">
                <h2 class="short-title">${short.name}</h2>
                <p class="short-description">${short.description}</p>
            </div>
            `
            const video = document.createElement("video");
            video.src = short.path;
            video.autoplay = true;
            video.muted = true;
            video.loop = true;
            video.setAttribute("playsinline", ""); 
            video.setAttribute("controls", "");  

            if (short.caption_path) {
                const track = document.createElement("track");
                track.src = short.caption_path;
                track.kind = "captions";
                track.srclang = "en";
                track.default = true;
                video.appendChild(track);
            }
            div.appendChild(video)

            container.appendChild(div);
            currentIndex = (currentIndex + 1) % allShorts.length;
        }

        observeLastShort();
    }

    function observeLastShort() {
        const shorts = document.querySelectorAll(".short");
        const last = shorts[shorts.length - 1];

        const observer = new IntersectionObserver((entries) => {
            if (entries[0].isIntersecting && !loading) {
                loading = true;
                setTimeout(() => {
                    appendShorts(2);
                    loading = false;
                }, 200);
            }
        }, { threshold: 0.8 });

        observer.observe(last);
    }

    fetchAllShorts();

    fetch("/header")
        .then(res => res.text())
        .then(html => document.getElementById("header").innerHTML = html);

    fetch("/footer")
        .then(res => res.text())
        .then(html => document.getElementById("footer").innerHTML = html)

});