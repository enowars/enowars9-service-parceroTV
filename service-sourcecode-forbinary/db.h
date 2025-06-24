#include <sqlite3.h>
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>


bool username_exists(sqlite3 *db, const char *username);
char *get_user_password(sqlite3 *db, const char *username);

//Get functions
char* get_most_clicked_video_from_db(sqlite3 *db);
char* get_most_liked_video_from_db(sqlite3 *db);
char* get_most_disliked_video_from_db(sqlite3 *db);
char* get_most_commented_video_from_db(sqlite3 *db);
char* get_comments_of_video_from_db(sqlite3 *db, const char *video_id);
char* get_video_stats_from_db(sqlite3 *db, const char *video_id);
char* get_user_stats_from_db(sqlite3 *db, const char *username);
char* get_user_videos_from_db(sqlite3 *db, const char *username);
char* get_recently_added_videos_from_db(sqlite3 *db);
char* get_public_playlists_from_db(sqlite3 *db);