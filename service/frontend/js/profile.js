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
});