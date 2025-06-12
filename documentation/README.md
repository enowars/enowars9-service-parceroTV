## Documentation
# Flagstore
1. Video description if the video is set private

# Vulnerability and Exploit
1. Hash-Collision
The video path is calculated by creating a hash which uses metadata from the video.
    h(title, artist, genre) = hash
The data is leaked, by looking at the Profile of the user (attack.json)
Using 
ffmpeg -i input.mp4 -metadata title="Leaked_title" -metadata artist="username" -metadata genre="leaked_genre" output.mp4
one can create a video that has the same metadata, which will lead to a hash collision.
When now clicking at the video the first uploaded video with this path is displayed.



