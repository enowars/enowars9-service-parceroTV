from asyncio import StreamReader, StreamWriter
import asyncio
import random
import string
from time import sleep
import faker
import os
from httpx import AsyncClient, Response
from names import adjectives, content_types, cities, countries, german_proverbs
import ffmpeg
from bs4 import BeautifulSoup
import subprocess
import tempfile
from pathlib import Path
from exploit_translation import full_exploit_for_shorts, extract_vtt_words

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
CLI_PORT = 7778
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
    if status_code in [303]:
        logger.info(f"user:{username} successfully registered with content to {response.text}")
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
    if status_code in [303] and response.headers.get("Location") == "/app/home":
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

def generate_short_title() -> str:
    """Generate a random title for a short video."""
    adj = random.choice(adjectives)
    content = random.choice(content_types)
    number = random.randint(1, 1000000)
    return f"short-{adj}-{content}-{number}"

def generate_short_description() -> str:
    """Generate a random description for a short video."""
    return f"{random.choice(german_proverbs)} {random.randint(1, 10)}"

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

def get_duration(filename):
    result = subprocess.run(
        ["ffprobe", "-v", "error", "-show_entries",
         "format=duration", "-of",
         "default=noprint_wrappers=1:nokey=1", filename],
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT
    )
    return float(result.stdout)

def get_duration_from_bytes(video_bytes):
    with tempfile.NamedTemporaryFile(suffix=".mp4") as tmp:
        tmp.write(video_bytes)
        tmp.flush()  # Ensure it's written before probing

        result = subprocess.run(
            ["ffprobe", "-v", "error", "-show_entries",
             "format=duration", "-of", "default=noprint_wrappers=1:nokey=1", tmp.name],
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT
        )
        return float(result.stdout)

async def upload_short(client: AsyncClient, logger, short_title, description, subtitles, translate_to_spanish):
    """Upload a short video with the given title, description, subtitles and translation option."""
    logger.info(f"Uploading short video with title: {short_title}, description: {description}, subtitles: {subtitles}, translate_to_spanish: {translate_to_spanish}")
    
    with open(get_random_video_path(), "rb") as video_file:
        duration = get_duration(video_file.name)
        duration = round(duration, 2)  # Round to 2 decimal places Like in Client javascript
        logger.info(f"Video duration: {duration} seconds, {video_file.name}")
        files = {
            "name": (None, short_title),
            "description": (None, description),
            "captions": (None, subtitles),
            "translate_to_spanish": (None, str(translate_to_spanish).lower()),
            "file": (Path(video_file.name).name, video_file, "video/mp4"),
            "duration": (None, str(duration)),
        }
        
        response = await client.post("/app/create_short", files=files)
    
    status_code = response.status_code
    if status_code == 404:
        logger.info(f"Client error {status_code} with {response.text}")
   
    if status_code in [303]:
        logger.info(f"Short video was successfully uploaded")
        redirect_url = response.headers.get("Location")
        logger.info(f"Redirected to: {redirect_url}")
        return redirect_url
    else:
        raise MumbleException(f"Failed to upload short video with title {short_title}, status code: {status_code}, and response: {response.text}.")

async def get_video_bytes_from_short(short, client: AsyncClient, logger: LoggerAdapter) -> bytes:
    video_path = short.get("path")
    video = await client.get(video_path)
    short_list = await client.get("/videos")
    if video.status_code != 200:
        logger.error(f"Failed to get video bytes from {video_path}, status code: {video.status_code}")
        raise MumbleException(f"Failed to get video bytes from {video_path}")
    
    logger.info(f"Successfully retrieved video bytes from {video_path}")
    logger.info(f"videos list response: {short_list.text}, with status code {short_list.status_code}, and url {short_list.url}")
    return video.content

def get_short_to_exploit(shorts, short_title, logger):
    for short in shorts:
        if short.get("name") == short_title:
            logger.info(f"Found short with title {short_title}")
            return short
    raise MumbleException(f"Short with title {short_title} not found in response")
    
"""
CHECKER FUNCTIONS
"""

## PUTFLAG AND GETFLAG FUNCTIONS

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
        

@checker.putflag(1)
async def putflag_short(
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
    
    short_title = generate_short_title()
    description = generate_short_description()
    subtitles = task.flag
    translate_to_spanish = True
    
    await upload_short(client, logger, short_title, description, subtitles, translate_to_spanish)

    await db.set("userdata2", (username, password, short_title))
    logger.info(f"User data saved for {username}, and uploaded successfully")
    
    return short_title

@checker.getflag(1)
async def getflag_short(
    task: GetflagCheckerTaskMessage, db: ChainDB, logger: LoggerAdapter, client: AsyncClient
) -> None:
    try:
        username, password, short_title = await db.get("userdata2")
    except KeyError:
        raise MumbleException("Missing database entry from putflag")
    
    logger.debug(f"getflag(1) for the user {username} and short {short_title} flag should be {task.flag}")
    await login(client, username, password,logger)
    
    logger.info(f"try to access flag in /app/get_shorts")
    response = await client.get("/get_shorts")
    try:
        json = response.json()
    except ValueError:
        logger.error("Failed to parse JSON response")
        raise MumbleException("Failed to parse JSON response from /get_shorts")

    logger.debug("response.json is: " + str(json))
    
    # Check if the short with the title exists
    for short in json:
        #logger.info(f"Checking short: {short.get('title')}")
        if short.get("name") == short_title:
            logger.info(f"Found short with title {short_title}")
            assert_in(task.flag, short.get("original_captions"), "Flag missing")
            return
    
    raise MumbleException(f"Short with title {short_title} not found in response")

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
    
    
@checker.putnoise(2)
async def putnoise_short_description(task: PutnoiseCheckerTaskMessage, db: ChainDB, logger: LoggerAdapter, client: AsyncClient):
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

    short_title = generate_short_title()
    description = generate_short_description()
    subtitles = "This is a test subtitle"
    translate_to_spanish = False
    
    await upload_short(client, logger, short_title, description, subtitles, translate_to_spanish)
    await db.set('information3', (username, password, short_title, description))

@checker.getnoise(2)
async def getnoise_short_description(task: GetnoiseCheckerTaskMessage, db: ChainDB, logger: LoggerAdapter, client: AsyncClient):
    logger.info(f"Getnoise 2 (short description) {client.base_url}")
    try:
        username, password, short_title, description = await db.get('information3')
    except:
        logger.error("Putnoise 2 (short description) failed : DB couldnt get information")
        raise MumbleException('Putnoise(2) failed')
    
    await login(client, username, password, logger)
    
    response = await client.get("/get_shorts")
    try:
        json = response.json()
    except ValueError:
        logger.error("Failed to parse JSON response from /get_shorts")
        raise MumbleException("Failed to parse JSON response from /get_shorts")

    logger.debug("response.json is: " + str(json))
    
    for short in json:
        if short.get("name") == short_title:
            logger.info(f"Found short with title {short_title}")
            assert_in(description, short.get("description"), "Description is wrong or missing")
            return
    
    raise MumbleException(f"Short with title {short_title} not found in response")


@checker.havoc(0)
async def havoc_failed_login(task: HavocCheckerTaskMessage, logger: LoggerAdapter, client: AsyncClient):
    username: str = "".join(
        random.choices(string.ascii_uppercase + string.digits, k=12)
    )
    password: str = "".join(
        random.choices(string.ascii_uppercase + string.digits, k=12)
    )

    try:
        failed_login = await login(client, username, password, logger)
    except MumbleException as e:
        logger.info(f"Failed login for user {username} with password {password} raised MumbleException: {e} like expected")
        return
    
    raise MumbleException(f"Failed to get unauthorized response for user {username} with password {password}")
    
    logger.info(f"havoc(0) FAILED login worked fine for user {username} with password {password}")

@checker.havoc(1)
async def havoc_get_logo(task: HavocCheckerTaskMessage, logger: LoggerAdapter, client: AsyncClient):
    logger.info("havoc(1) get logo")
    response = await client.get("/assets/ParcerroTV.svg")
    if response.status_code != 200:
        raise MumbleException(f"Failed to get ParceroTV svg logo, status code: {response.status_code}")
    
    logger.info("havoc(1) get logo worked fine")
    assert_in("<svg", response.text, "Logo SVG not found in response")

@checker.havoc(2)
async def havoc_get_playlist(task: HavocCheckerTaskMessage, logger: LoggerAdapter, client: AsyncClient):
    logger.info("havoc(2) get playlist")
    pass

@checker.havoc(3)
async def havoc_get_correct_vtt(task: HavocCheckerTaskMessage, logger: LoggerAdapter, client: AsyncClient):
    logger.info("havoc(3) get correct vtt")
    username: str = "".join(
        random.choices(string.ascii_uppercase + string.digits, k=12)
    )
    password: str = "".join(
        random.choices(string.ascii_uppercase + string.digits, k=12)
    )
    await signup(client, username, password, logger)
    await login(client, username, password, logger)
    
    short_title = generate_short_title()
    description = generate_short_description()
    subtitles = "Que chimba sog que chimba"
    translate_to_spanish = False
    
    await upload_short(client, logger, short_title, description, subtitles, translate_to_spanish)
    
    response = await client.get("/get_shorts")
    json = response.json()
    logger.info(f"Shorts response: {json}")
    
    for short in json:
        if short.get("name") == short_title:
            caption_path = short.get("caption_path")
            captions = await client.get(caption_path)
            if captions.status_code != 200:
                raise MumbleException(f"Failed to get captions for the short {short_title}, status code: {captions.status_code}")
            logger.info(f"Captions for the short {short_title} are {captions.text}")
            assert_in("Que chimba sog que chimba", ' '.join(extract_vtt_words(captions.text)), "Captions not found in response")
            return
    raise MumbleException(f"Short with title {short_title} not found in response, cannot get captions")

@checker.havoc(4)
async def havoc_same_text_same_translation(task: HavocCheckerTaskMessage, logger: LoggerAdapter, client: AsyncClient):
    logger.info("havoc(4) same text same translation")
    username: str = "".join(
        random.choices(string.ascii_uppercase + string.digits, k=12)
    )
    password: str = "".join(
        random.choices(string.ascii_uppercase + string.digits, k=12)
    )
    
    await signup(client, username, password, logger)
    await login(client, username, password, logger)
    
    short_title = generate_short_title()
    short_title2 = generate_short_title()  # Generate a different title for the second short
    description = generate_short_description()
    subtitles = "".join(
        random.choices(string.ascii_uppercase + string.digits, k=12)
    )
    translate_to_spanish = True
    await upload_short(client, logger, short_title, description, subtitles, translate_to_spanish)
    await upload_short(client, logger, short_title2, description, subtitles, translate_to_spanish)
    
    response = await client.get("/get_shorts")
    json = response.json()
    logger.info(f"Shorts response: {json}")
    for short in json:
        if short.get("name") == short_title:
            caption_path = short.get("caption_path")
            captions = await client.get(caption_path)
            if captions.status_code != 200:
                raise MumbleException(f"Failed to get captions for the short {short_title}, status code: {captions.status_code}")
            
        if short.get("name") == short_title2:
            caption_path2 = short.get("caption_path")
            captions2 = await client.get(caption_path2)
            if captions2.status_code != 200:
                raise MumbleException(f"Failed to get captions for the short {short_title2}, status code: {captions2.status_code}")
    
    if captions.text != captions2.text:
        raise MumbleException(f"Captions(vtt) for the shorts {short_title} and {short_title2} are different, but they should be the same")

@checker.havoc(5)
async def havoc_words_in_translation_array(task: HavocCheckerTaskMessage, logger: LoggerAdapter, client: AsyncClient):
    pass

@checker.havoc(6)
async def havoc_get_vtt_index(task: HavocCheckerTaskMessage, logger: LoggerAdapter, client: AsyncClient):
    username_attacker: str = "".join(
        random.choices(string.ascii_uppercase + string.digits, k=12)
    )
    password_attacker: str = "".join(
        random.choices(string.ascii_uppercase + string.digits, k=12)
    )
    
    await signup(client, username_attacker, password_attacker, logger)
    await login(client, username_attacker, password_attacker, logger)
    
    vtts = await client.get("/vtt")
    if vtts.status_code != 200:
        raise MumbleException("Failed to get VTTs")
    logger.info(f"VTTs response: {vtts.text}")
    assert_in("vtt", vtts.text, "VTTs not found in response")

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



@checker.exploit(1)
async def exploit_short(task: ExploitCheckerTaskMessage, searcher: FlagSearcher, client: AsyncClient, logger:LoggerAdapter) -> Optional[str]:
    username_attacker: str = "".join(
        random.choices(string.ascii_uppercase + string.digits, k=12)
    )
    password_attacker: str = "".join(
        random.choices(string.ascii_uppercase + string.digits, k=12)
    )
    
    await signup(client, username_attacker, password_attacker, logger)
    await login(client, username_attacker, password_attacker, logger)
    
    logger.info(f"Exploiting {task.attack_info} variant {task.variant_id} task {task}")
    short_title = task.attack_info
    response = await client.get(f"/get_shorts")
    if response.status_code != 200:
        raise MumbleException()
    
    try:
        shorts = response.json()
    except ValueError:
        logger.error("Failed to parse JSON response from /get_shorts")
        raise MumbleException("Failed to parse JSON response from /get_shorts")

    short_to_exploit = get_short_to_exploit(shorts, short_title, logger)
    video_bytes = await get_video_bytes_from_short(short_to_exploit, client, logger)
    duration = get_duration_from_bytes(video_bytes)
    duration_two_decimal = round(duration, 2)
    logger.info(f"Duration of the video title {short_title} is {duration_two_decimal} seconds")
    caption_path = short_to_exploit.get("caption_path")
    captions = await client.get(caption_path)
    vtts = await client.get("/vtt")
    captions_vtt = captions.text
    vtts_vtt = vtts.text
    logger.info(f"Captions for the short {short_title} are {captions_vtt} from the path {caption_path} with response {captions}, url is {captions.url}")
    logger.info(f"VTTs for the short {short_title} are {vtts_vtt} from the path {vtts.url}")

    flag = full_exploit_for_shorts(duration_two_decimal, captions_vtt)
    logger.info(f"Flag for short {short_title} is {flag}")
    
    if flag := searcher.search_flag(flag):
        logger.info(f"Flag found: {flag}")
        return flag
    else:
        logger.error("Flag not found in the exploited short")
        raise MumbleException("Flag not found in the exploited short")




if __name__ == "__main__":
    checker.run()