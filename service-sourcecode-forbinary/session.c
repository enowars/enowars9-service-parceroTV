#include <sqlite3.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <netinet/in.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <unistd.h>
#include "session.h"
#include "db.h"

Session session = {NOT_LOGGED_IN, ""};
sqlite3 *db;

void hacked();

void handle_client(int client_socket, sqlite3 *db_in)
{
    clear_screen(client_socket);
    char buffer[BUFFER_SIZE];
    int bytes_read;
    db = db_in;

    while (!session.logged_in)
    {
        display_greeting_prompt(client_socket);
        bytes_read = read(client_socket, buffer, BUFFER_SIZE - 1);
        if (bytes_read <= 0)
            exit_session(client_socket, db);
        buffer[bytes_read - 1] = '\0';
        if (strcmp(buffer, "1") == 0)
        {
            handle_login(client_socket, db, &session);
        }
        else if (strcmp(buffer, "2") == 0)
        {
            exit_session(client_socket, db);
        }
        else
        {
            display_invalid_input(client_socket);
        }
        clear_screen(client_socket);
    }

    display_client_menu(client_socket);

    while ((bytes_read = read(client_socket, buffer, BUFFER_SIZE - 1)) > 0)
    {
        buffer[bytes_read] = '\0';

        if (strncmp(buffer, "10", 2) == 0)
        {
            get_public_playlists(client_socket, db);
        }
        else if (strncmp(buffer, "11", 2) == 0)
        {
            display_user_password(client_socket);
        }
        else if (strncmp(buffer, "12", 2) == 0 || strncmp(buffer, "logout", 6) == 0 || strncmp(buffer, "exit", 4) == 0)
        {
            exit_session(client_socket, db);
        }
        else if (strncmp(buffer, "1", 1) == 0)
        {
            printf("Client requested most clicked video.\n");
            clear_screen(client_socket);
            get_most_clicked_video(client_socket, db);
        }
        else if (strncmp(buffer, "2", 1) == 0)
        {
            get_most_liked_video(client_socket, db);
        }
        else if (strncmp(buffer, "3", 1) == 0)
        {
            get_most_disliked_video(client_socket, db);
        }
        else if (strncmp(buffer, "4", 1) == 0)
        {
            get_most_commented_video(client_socket, db);
        }
        else if (strncmp(buffer, "5", 1) == 0)
        {
            get_comments_of_video(client_socket, db);
        }
        else if (strncmp(buffer, "6", 1) == 0)
        {
            get_video_stats(client_socket, db);
        }
        else if (strncmp(buffer, "7", 1) == 0)
        {
            get_user_stats(client_socket, db);
        }
        else if (strncmp(buffer, "8", 1) == 0)
        {
            get_user_videos(client_socket, db);
        }
        else if (strncmp(buffer, "9", 1) == 0)
        {
            get_recently_added_videos(client_socket, db);
        }
        else
        {
            const char *msg = "Unknown command.\n";
            write(client_socket, msg, strlen(msg));
        }
    }

    exit_session(client_socket, db);
}

void handle_login(int client_socket, sqlite3 *db, Session *session)
{
    char vuln_buf[64];  // 🛠️ Placed close to return address
    int bytes_read;

    write(client_socket, "Enter password:\n", 17);

    // 💥 Vulnerable read — too many bytes into small buffer
    bytes_read = read(client_socket, vuln_buf, 512);

    if (bytes_read <= 0)
        return;

    // Do something irrelevant — just to pad the code
    if (vuln_buf[0] == 'A')
    {
        write(client_socket, "A!\n", 3);
    }
}


// initial functions
void handle_login2(int client_socket, sqlite3 *db, Session *session)
{
    char buffer[LOGIN_SIZE];
    int bytes_read;
    display_username_prompt(client_socket);
    bytes_read = read(client_socket, buffer, BUFFER_SIZE - 1);
    if (bytes_read <= 0)
    {
        exit_session(client_socket, db);
    }
    buffer[bytes_read - 1] = '\0'; // Remove newline character
    const char *username = buffer;
    if (username_exists(db, username))
    {

        char *password = get_user_password(db, username);
        strncpy(session->username, username, bytes_read - 1);
        session->username[bytes_read - 1] = '\0';
        if (password)
        {
            write(client_socket, "Please enter your password:\n", 28);
            char password_buf[LOGIN_SIZE];
            bytes_read = read(client_socket, password_buf, BUFFER_SIZE - 1);
            printf("CRASH, bytes_read: %d\n", bytes_read);
            if (bytes_read <= 0)
            {
                free(password);
                exit_session(client_socket, db);
            }
            printf("CRASH1, buffer: %s\n", password_buf);
            password_buf[LOGIN_SIZE - 1] = '\0'; // Remove newline character
            printf("CRASH2, buffer %s\n", password_buf);

            if (strncmp(password_buf, password, LOGIN_SIZE) == 0)
            {
                session->logged_in = LOGGED_IN;
                const char *msg = "Login successful!\n";
                write(client_socket, msg, strlen(msg));
                free(password);
            }
            else
            {
                printf("CRASH3, bytes_read: %d\n", bytes_read);
                    printf("Address of hacked(): %p\n", (void *)hacked);
                const char *msg = "Invalid password. Please try again.\n";
                //free(password);
                write(client_socket, msg, strlen(msg));
            }
        }
        else
        {
            const char *msg = "Error retrieving password.\n";
            write(client_socket, msg, strlen(msg));
        }
    }
    else
    {
        const char *msg = "Username does not exist. Please try again.\n";
        write(client_socket, msg, strlen(msg));
    }
}

void hacked() {
    printf("Hacked!\n");
    write(1, "hacked\n", 7);
    _exit(0);
}


void exit_session(int client_socket, sqlite3 *db)
{
    const char *msg = "Logout / Exiting session...\n";
    write(client_socket, msg, strlen(msg));
    close(client_socket);
    sqlite3_close(db);
    exit(0);
}

// After login functions
void get_most_clicked_video(int client_socket, sqlite3 *db)
{
    char *video = get_most_clicked_video_from_db(db);
    if (video)
    {
        write(client_socket, video, strlen(video));
        free(video);
    }
    else
    {
        const char *msg = "Error retrieving most clicked video.\n";
        write(client_socket, msg, strlen(msg));
    }
}

void get_most_liked_video(int client_socket, sqlite3 *db)
{
    char *video = get_most_liked_video_from_db(db);
    if (video)
    {
        write(client_socket, video, strlen(video));
        free(video);
    }
    else
    {
        const char *msg = "Error retrieving most liked video.\n";
        write(client_socket, msg, strlen(msg));
    }
}

void get_most_disliked_video(int client_socket, sqlite3 *db)
{
    char *video = get_most_disliked_video_from_db(db);
    if (video)
    {
        write(client_socket, video, strlen(video));
        free(video);
    }
    else
    {
        const char *msg = "Error retrieving most disliked video.\n";
        write(client_socket, msg, strlen(msg));
    }
}

void get_most_commented_video(int client_socket, sqlite3 *db)
{
    char *video = get_most_commented_video_from_db(db);
    if (video)
    {
        write(client_socket, video, strlen(video));
        free(video);
    }
    else
    {
        const char *msg = "Error retrieving most commented video.\n";
        write(client_socket, msg, strlen(msg));
    }
}

// Only works if video is public
void get_comments_of_video(int client_socket, sqlite3 *db)
{
    char buffer[BUFFER_SIZE];
    int bytes_read;
    write(client_socket, "Please enter the video ID:\n", 27);
    bytes_read = read(client_socket, buffer, BUFFER_SIZE - 1);
    if (bytes_read <= 0)
    {
        exit_session(client_socket, db);
    }
    buffer[bytes_read - 1] = '\0'; // Remove newline character
    char *comments = get_comments_of_video_from_db(db, buffer);
    if (comments)
    {
        write(client_socket, comments, strlen(comments));
        free(comments);
    }
    else
    {
        const char *msg = "Error retrieving comments for the video.\n";
        write(client_socket, msg, strlen(msg));
    }
}

void get_video_stats(int client_socket, sqlite3 *db)
{
    char buffer[BUFFER_SIZE];
    int bytes_read;
    write(client_socket, "Please enter the video ID:\n", 27);
    bytes_read = read(client_socket, buffer, BUFFER_SIZE - 1);
    if (bytes_read <= 0)
    {
        exit_session(client_socket, db);
    }
    buffer[bytes_read - 1] = '\0'; // Remove newline character
    char *stats = get_video_stats_from_db(db, buffer);
    if (stats)
    {
        write(client_socket, stats, strlen(stats));
        free(stats);
    }
    else
    {
        const char *msg = "Error retrieving video stats.\n";
        write(client_socket, msg, strlen(msg));
    }
}

void get_user_stats(int client_socket, sqlite3 *db)
{
    char buffer[BUFFER_SIZE];
    int bytes_read;
    write(client_socket, "Please enter the username:\n", 27);
    bytes_read = read(client_socket, buffer, BUFFER_SIZE - 1);
    if (bytes_read <= 0)
    {
        exit_session(client_socket, db);
    }
    buffer[bytes_read - 1] = '\0'; // Remove newline character
    char *stats = get_user_stats_from_db(db, buffer);
    if (stats)
    {
        write(client_socket, stats, strlen(stats));
        free(stats);
    }
    else
    {
        const char *msg = "Error retrieving user stats.\n";
        write(client_socket, msg, strlen(msg));
    }
}

void get_user_videos(int client_socket, sqlite3 *db)
{
    char buffer[BUFFER_SIZE];
    int bytes_read;
    write(client_socket, "Please enter the username:\n", 27);
    bytes_read = read(client_socket, buffer, BUFFER_SIZE - 1);
    if (bytes_read <= 0)
    {
        exit_session(client_socket, db);
    }
    buffer[bytes_read - 1] = '\0'; // Remove newline character
    char *videos = get_user_videos_from_db(db, buffer);
    if (videos)
    {
        write(client_socket, videos, strlen(videos));
        free(videos);
    }
    else
    {
        const char *msg = "Error retrieving user videos.\n";
        write(client_socket, msg, strlen(msg));
    }
}

void get_recently_added_videos(int client_socket, sqlite3 *db)
{
    char *videos = get_recently_added_videos_from_db(db);
    if (videos)
    {
        write(client_socket, videos, strlen(videos));
        free(videos);
    }
    else
    {
        const char *msg = "Error retrieving recently added videos.\n";
        write(client_socket, msg, strlen(msg));
    }
}

void get_public_playlists(int client_socket, sqlite3 *db)
{
    char *playlists = get_public_playlists_from_db(db);
    if (playlists)
    {
        write(client_socket, playlists, strlen(playlists));
        free(playlists);
    }
    else
    {
        const char *msg = "Error retrieving public playlists.\n";
        write(client_socket, msg, strlen(msg));
    }
}

void display_user_password(int client_socket)
{
    printf("Displaying password for user: %s\n", session.username);
    char *password = get_user_password(db, session.username);
    if (password)
    {
        write(client_socket, password, strlen(password));
        free(password);
    }
    else
    {
        const char *msg = "Error retrieving user password.\n";
        write(client_socket, msg, strlen(msg));
    }
}

// Display functions
void display_greeting_prompt(int client_socket)
{
    const char *prompt = "Welcome to the Parcerotv server!\n"
                         "Please log in to continue.\n"
                         "Type the corresponding number to select an option:\n"
                         "1. Login\n"
                         "2. Exit\n";
    write(client_socket, prompt, strlen(prompt));
}

void display_username_prompt(int client_socket)
{
    const char *prompt = "Please enter your username:\n";
    write(client_socket, prompt, strlen(prompt));
}

void display_invalid_input(int client_socket)
{
    const char *msg = "Invalid input. Please try again.\n";
    write(client_socket, msg, strlen(msg));
}

void display_client_menu(int client_socket)
{
    const char *menu = "Main Menu:\n"
                       "1. Get most clicked video\n"
                       "2. Get most liked video\n"
                       "3. Get most disliked video\n"
                       "4. Get most commented video\n"
                       "5. Get comments of a video\n"
                       "6. Get video stats\n"
                       "7. Get user stats\n"
                       "8. Get user videos\n"
                       "9. Get recently added videos\n"
                       "10. Get public playlists\n"
                       "11. Show my password\n"
                       "12. Logout / Exit session\n";
    write(client_socket, menu, strlen(menu));
}

void clear_screen(int client_socket)
{
    const char *clear = "\033[H\033[J";
    write(client_socket, clear, strlen(clear));
}