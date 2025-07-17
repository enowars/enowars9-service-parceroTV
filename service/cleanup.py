import sqlite3
import time
import datetime
from pathlib import Path

DELETE_TIMEINTEVALL = 15

def delete_old_files(directory: str, age_minutes: int = 15):
    """
    Delete all files in `directory` older than `age_minutes`.
    """
    cutoff = time.time() - age_minutes * 60
    dir_path = Path(directory)

    for path in dir_path.iterdir():
        if path.is_file():
            mtime = path.stat().st_mtime
            if mtime < cutoff:
                try:
                    path.unlink()
                    print(f"Deleted: {path}")
                except Exception as e:
                    print(f"Error deleting {path}: {e}")

while True:
    connection = sqlite3.connect("/service/data/parcerotv.db")
    cursor = connection.cursor()

    cutoff_time = datetime.datetime.now(datetime.timezone.utc) - datetime.timedelta(minutes=DELETE_TIMEINTEVALL)
    
    cursor.execute("DELETE FROM users WHERE created_at < ?", (cutoff_time.strftime("%Y-%m-%d %H:%M:%S"), ))
    cursor.execute("DELETE FROM videos WHERE created_at < ?", (cutoff_time.strftime("%Y-%m-%d %H:%M:%S"), ))
    cursor.execute("DELETE FROM comments WHERE created_at < ?", (cutoff_time.strftime("%Y-%m-%d %H:%M:%S"), ))
    cursor.execute("DELETE FROM shorts WHERE created_at < ?", (cutoff_time.strftime("%Y-%m-%d %H:%M:%S"), ))
    cursor.execute("DELETE FROM playlist WHERE created_at < ?", (cutoff_time.strftime("%Y-%m-%d %H:%M:%S"), ))
    cursor.execute("DELETE FROM has_liked WHERE created_at < ?", (cutoff_time.strftime("%Y-%m-%d %H:%M:%S"), ))
    cursor.execute("DELETE FROM access_rights_playlist WHERE created_at < ?", (cutoff_time.strftime("%Y-%m-%d %H:%M:%S"), ))
    cursor.execute("DELETE FROM videos_in_playlist WHERE created_at < ?", (cutoff_time.strftime("%Y-%m-%d %H:%M:%S"), ))
    
    #Delete Files in directory
    delete_old_files("/service/data/videos/", DELETE_TIMEINTEVALL)
    delete_old_files("/service/data/thumbnails/", DELETE_TIMEINTEVALL)
    delete_old_files("/service/data/private/", DELETE_TIMEINTEVALL)
    delete_old_files("/service/data/shorts/", DELETE_TIMEINTEVALL)
    delete_old_files("/service/data/vtt/", DELETE_TIMEINTEVALL)
    connection.commit()
    connection.close()

    time.sleep(60)