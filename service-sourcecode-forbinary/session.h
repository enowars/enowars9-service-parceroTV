
#include <sqlite3.h>


#define BUFFER_SIZE 1024
#define LOGIN_SIZE 64
#define NOT_LOGGED_IN 0
#define LOGGED_IN 1

typedef struct session
{
    int logged_in;
    char username[LOGIN_SIZE];
} Session;


//Main Loop
void handle_client(int client_socket, sqlite3 *db);

// Initial functions
void exit_session(int client_socket,sqlite3 *db);
void handle_login(int client_socket, sqlite3 *db, Session *session);

//After login functions
void get_most_clicked_video(int client_socket, sqlite3 *db);
void get_most_liked_video(int client_socket, sqlite3 *db);
void get_most_disliked_video(int client_socket, sqlite3 *db);
void get_most_commented_video(int client_socket, sqlite3 *db);

void get_comments_of_video(int client_socket, sqlite3 *db);
void get_video_stats(int client_socket, sqlite3 *db);
void get_user_stats(int client_socket, sqlite3 *db);
void get_user_videos(int client_socket, sqlite3 *db);

void get_recently_added_videos(int client_socket, sqlite3 *db);
void get_public_playlists(int client_socket, sqlite3 *db);

void display_user_password(int client_socket);


//Display functions
void display_greeting_prompt(int client_socket);
void display_username_prompt(int client_socket);
void display_invalid_input(int client_socket);
void display_client_menu(int client_socket);
void display_login_prompt(int client_socket);
void clear_screen(int client_socket);
