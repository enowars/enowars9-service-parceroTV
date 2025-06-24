#include "db.h" 
#include <sqlite3.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

bool username_exists(sqlite3 *db, const char *username) {
    printf("Checking if username '%s', with len %lu exists in the database...\n", username, strlen(username));
    sqlite3_stmt *stmt;
    const char *sql = "SELECT COUNT(*) FROM users WHERE name = ?";

    if (sqlite3_prepare_v2(db, sql, -1, &stmt, NULL) != SQLITE_OK) {
        fprintf(stderr, "Failed to prepare statement: %s\n", sqlite3_errmsg(db));
        return false;
    }

    sqlite3_bind_text(stmt, 1, username, -1, SQLITE_STATIC);

    bool exists = false;
    if (sqlite3_step(stmt) == SQLITE_ROW) {
        exists = sqlite3_column_int(stmt, 0) > 0;
    }

    sqlite3_finalize(stmt);
    return exists;
}

//Gets password, alloc on heap
char *get_user_password(sqlite3 *db, const char *username) {
    sqlite3_stmt *stmt;
    const char *sql = "SELECT password FROM users WHERE name = ?";

    if (sqlite3_prepare_v2(db, sql, -1, &stmt, NULL) != SQLITE_OK) {
        fprintf(stderr, "Failed to prepare statement: %s\n", sqlite3_errmsg(db));
        return NULL;
    }

    sqlite3_bind_text(stmt, 1, username, -1, SQLITE_STATIC);

    char *password = NULL;
    if (sqlite3_step(stmt) == SQLITE_ROW) {
        const unsigned char *pwd = sqlite3_column_text(stmt, 0);
        password = strdup((const char *)pwd);
    }

    sqlite3_finalize(stmt);
    return password;
}

//Get functions
char* get_most_clicked_video_from_db(sqlite3 *db){
    sqlite3_stmt *stmt;
    const char *sql = "SELECT videoId, name FROM videos WHERE is_private = 0 ORDER BY clicks DESC LIMIT 1";

    if (sqlite3_prepare_v2(db, sql, -1, &stmt, NULL) != SQLITE_OK) {
        fprintf(stderr, "Failed to prepare statement: %s\n", sqlite3_errmsg(db));
        return NULL;
    }

    char *result = NULL;
    if (sqlite3_step(stmt) == SQLITE_ROW) {
        const unsigned char *vid = sqlite3_column_text(stmt, 0);
        const unsigned char *name = sqlite3_column_text(stmt, 1);
        size_t len = strlen((const char *)vid) + strlen((const char *)name) + 4;
        result = malloc(len);
        if (result == NULL) {
            fprintf(stderr, "Memory allocation failed\n");
            sqlite3_finalize(stmt);
            return NULL;
        }
        snprintf(result, len, "%s (%s)", vid, name);
    }
    sqlite3_finalize(stmt);
    return result;
}

char* get_most_liked_video_from_db(sqlite3 *db){
    sqlite3_stmt *stmt;
    const char *sql = "SELECT videoId, name FROM videos WHERE is_private = 0 ORDER BY likes DESC LIMIT 1";

    if (sqlite3_prepare_v2(db, sql, -1, &stmt, NULL) != SQLITE_OK) {
        fprintf(stderr, "Failed to prepare statement: %s\n", sqlite3_errmsg(db));
        return NULL;
    }

    char *message = NULL;
    if (sqlite3_step(stmt) == SQLITE_ROW) {
        const unsigned char *vid = sqlite3_column_text(stmt, 0);
        const unsigned char *name = sqlite3_column_text(stmt, 1);
        size_t len = strlen((const char *)vid) + strlen((const char *)name) + 4;
        message = malloc(len);
        if (message == NULL) {
            fprintf(stderr, "Memory allocation failed\n");
            sqlite3_finalize(stmt);
            return NULL; }
        snprintf(message, len, "%s (%s)", vid, name);
        
    }

    sqlite3_finalize(stmt);
    return message;
}

char* get_most_disliked_video_from_db(sqlite3 *db){
    sqlite3_stmt *stmt;
    const char *sql = "SELECT videoId, name FROM videos WHERE is_private = 0 ORDER BY dislikes DESC LIMIT 1";

    if (sqlite3_prepare_v2(db, sql, -1, &stmt, NULL) != SQLITE_OK) {
        fprintf(stderr, "Failed to prepare statement: %s\n", sqlite3_errmsg(db));
        return NULL;
    }

    char *message = NULL;
    if (sqlite3_step(stmt) == SQLITE_ROW) {
        const unsigned char *vid = sqlite3_column_text(stmt, 0);
        const unsigned char *name = sqlite3_column_text(stmt, 1);
        size_t len = strlen((const char *)vid) + strlen((const char *)name) + 4; // +4 for formatting
        message = malloc(len);
        if (message == NULL) {
            fprintf(stderr, "Memory allocation failed\n");
            sqlite3_finalize(stmt);
            return NULL;     }
        snprintf(message, len, "%s (%s)", vid, name);
        }
    sqlite3_finalize(stmt);
    return message;
}

char* get_most_commented_video_from_db(sqlite3 *db){
    sqlite3_stmt *stmt;
    const char *sql = 
        "SELECT videoId, name FROM videos "
        "WHERE is_private = 0 "
        "ORDER BY (SELECT COUNT(*) FROM comments WHERE comments.videoId = videos.videoId) DESC "
        "LIMIT 1";

    if (sqlite3_prepare_v2(db, sql, -1, &stmt, NULL) != SQLITE_OK) {
        fprintf(stderr, "Failed to prepare statement: %s\n", sqlite3_errmsg(db));
        return NULL;
    }

     char *message = NULL;
    if (sqlite3_step(stmt) == SQLITE_ROW) {
        const unsigned char *vid = sqlite3_column_text(stmt, 0);
        const unsigned char *name = sqlite3_column_text(stmt, 1);
        size_t len = strlen((const char *)vid) + strlen((const char *)name) + 4; // +4 for formatting
        message = malloc(len);
        if (message == NULL) {
            fprintf(stderr, "Memory allocation failed\n");
            sqlite3_finalize(stmt);
            return NULL;     }
        snprintf(message, len, "%s (%s)", vid, name);
        }
    sqlite3_finalize(stmt);
    return message;
}

char* get_comments_of_video_from_db(sqlite3 *db, const char *video_id){
    sqlite3_stmt *stmt;
    const char *sql = "SELECT comment FROM comments WHERE videoId = ?";

    if (sqlite3_prepare_v2(db, sql, -1, &stmt, NULL) != SQLITE_OK) {
        fprintf(stderr, "Failed to prepare statement: %s\n", sqlite3_errmsg(db));
        return NULL;
    }

    sqlite3_bind_text(stmt, 1, video_id, -1, SQLITE_STATIC);

    char *comments = NULL;
    size_t total_length = 0;
    while (sqlite3_step(stmt) == SQLITE_ROW) {
        const unsigned char *comment = sqlite3_column_text(stmt, 0);
        size_t len = strlen((const char *)comment);
        comments = realloc(comments, total_length + len + 2); // +2 for newline and null terminator
        if (comments == NULL) {
            fprintf(stderr, "Memory allocation failed\n");
            sqlite3_finalize(stmt);
            return NULL;
        }
        if (total_length > 0) {
            comments[total_length++] = '\n'; // Add newline between comments
        }
        strcpy(comments + total_length, (const char *)comment);
        total_length += len;
    }

    sqlite3_finalize(stmt);
    return comments;
}

char* get_video_stats_from_db(sqlite3 *db, const char *video_id){
    sqlite3_stmt *stmt;
    const char *sql = "SELECT clicks, likes, dislikes FROM videos WHERE videoId = ?";

    if (sqlite3_prepare_v2(db, sql, -1, &stmt, NULL) != SQLITE_OK) {
        fprintf(stderr, "Failed to prepare statement: %s\n", sqlite3_errmsg(db));
        return NULL;
    }

    sqlite3_bind_text(stmt, 1, video_id, -1, SQLITE_STATIC);

    char *stats = NULL;
    if (sqlite3_step(stmt) == SQLITE_ROW) {
        int clicks = sqlite3_column_int(stmt, 0);
        int likes = sqlite3_column_int(stmt, 1);
        int dislikes = sqlite3_column_int(stmt, 2);
        stats = malloc(100); // Allocate enough space for the stats string
        snprintf(stats, 100, "Clicks: %d\nLikes: %d\nDislikes: %d", clicks, likes, dislikes);
    }

    sqlite3_finalize(stmt);
    return stats;
}

char* get_user_stats_from_db(sqlite3 *db, const char *username){
    sqlite3_stmt *stmt;
    const char *sql = "SELECT count(clicks), count(likes), count(dislikes) "
                      "FROM videos WHERE userID = (SELECT userID FROM users WHERE name = ?)";

    if (sqlite3_prepare_v2(db, sql, -1, &stmt, NULL) != SQLITE_OK) {
        fprintf(stderr, "Failed to prepare statement: %s\n", sqlite3_errmsg(db));
        return NULL;
    }

    sqlite3_bind_text(stmt, 1, username, -1, SQLITE_STATIC);

    char *stats = NULL;
    if (sqlite3_step(stmt) == SQLITE_ROW) {
        int clicks = sqlite3_column_int(stmt, 0);
        int likes = sqlite3_column_int(stmt, 1);
        int dislikes = sqlite3_column_int(stmt, 2);
        stats = malloc(100); // Allocate enough space for the stats string
        snprintf(stats, 100, "Clicks: %d\nLikes: %d\nDislikes: %d", clicks, likes, dislikes);
    }

    sqlite3_finalize(stmt);
    return stats;
}

char* get_user_videos_from_db(sqlite3 *db, const char *username){
    sqlite3_stmt *stmt;
    const char *sql = "SELECT videoId, name FROM videos WHERE userID = (SELECT userID FROM users WHERE name = ?)";

    if (sqlite3_prepare_v2(db, sql, -1, &stmt, NULL) != SQLITE_OK) {
        fprintf(stderr, "Failed to prepare statement: %s\n", sqlite3_errmsg(db));
        return NULL;
    }

    sqlite3_bind_text(stmt, 1, username, -1, SQLITE_STATIC);

    char *videos = NULL;
    size_t total_length = 0;
    while (sqlite3_step(stmt) == SQLITE_ROW) {
        const unsigned char *vid = sqlite3_column_text(stmt, 0);
        const unsigned char *name = sqlite3_column_text(stmt, 1);
        size_t len = strlen((const char *)vid) + strlen((const char *)name) + 4;
        videos = realloc(videos, total_length + len + 1); 
        if (videos == NULL) {
            fprintf(stderr, "Memory allocation failed\n");
            sqlite3_finalize(stmt);
            return NULL;
        }
        if (total_length > 0) {
            videos[total_length++] = '\n'; 
        }
        snprintf(videos + total_length, len + 1, "%s (%s)", vid, name);
        total_length += len - 1; 
    }

    sqlite3_finalize(stmt);
    return videos;
}

char* get_recently_added_videos_from_db(sqlite3 *db){
    sqlite3_stmt *stmt;
    const char *sql = "SELECT videoId, name FROM videos WHERE is_private = 0 ORDER BY added_date DESC LIMIT 10";

    if (sqlite3_prepare_v2(db, sql, -1, &stmt, NULL) != SQLITE_OK) {
        fprintf(stderr, "Failed to prepare statement: %s\n", sqlite3_errmsg(db));
        return NULL;
    }

    char *videos = NULL;
    size_t total_length = 0;
    while (sqlite3_step(stmt) == SQLITE_ROW) {
        const unsigned char *vid = sqlite3_column_text(stmt, 0);
        const unsigned char *name = sqlite3_column_text(stmt, 1);
        size_t len = strlen((const char *)vid) + strlen((const char *)name) + 4; // +4 for formatting
        videos = realloc(videos, total_length + len + 1); 
        if (videos == NULL) {
            fprintf(stderr, "Memory allocation failed\n");
            sqlite3_finalize(stmt);
            return NULL;
        }
        if (total_length > 0) {
            videos[total_length++] = '\n'; 
        }
        snprintf(videos + total_length, len + 1, "%s (%s)", vid, name);
        total_length += len - 1; 
    }

    sqlite3_finalize(stmt);
    return videos;
}

char* get_public_playlists_from_db(sqlite3 *db){
    sqlite3_stmt *stmt;
    const char *sql = "SELECT playlistId, name FROM playlists WHERE is_private = 0";

    if (sqlite3_prepare_v2(db, sql, -1, &stmt, NULL) != SQLITE_OK) {
        fprintf(stderr, "Failed to prepare statement: %s\n", sqlite3_errmsg(db));
        return NULL;
    }

    char *playlists = NULL;
    size_t total_length = 0;
    while (sqlite3_step(stmt) == SQLITE_ROW) {
        const unsigned char *pid = sqlite3_column_text(stmt, 0);
        const unsigned char *name = sqlite3_column_text(stmt, 1);
        size_t len = strlen((const char *)pid) + strlen((const char *)name) + 4; // +4 for formatting
        playlists = realloc(playlists, total_length + len + 1); 
        if (playlists == NULL) {
            fprintf(stderr, "Memory allocation failed\n");
            sqlite3_finalize(stmt);
            return NULL;
        }
        if (total_length > 0) {
            playlists[total_length++] = '\n'; 
        }
        snprintf(playlists + total_length, len + 1, "%s (%s)", pid, name);
        total_length += len - 1; 
    }

    sqlite3_finalize(stmt);
    return playlists;
}