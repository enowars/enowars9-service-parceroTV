#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <signal.h>
#include <netinet/in.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <sqlite3.h>
#include "signals.h"
#include "session.h"
#include "db.h"


#define PORT 12345
#define DB_FILE "parcerotv.db"
#define IS_CHILD(pid) (pid == 0)
#define IS_PARENT(pid) (pid > 0)





int main() {
    int server_fd, new_socket;
    struct sockaddr_in address;
    socklen_t addrlen = sizeof(address);

    // Set up signal handlers
    struct sigaction sa;
    sa.sa_handler = handle_sigchld;
    sigemptyset(&sa.sa_mask);
    sa.sa_flags = SA_RESTART | SA_NOCLDSTOP; //Resume interrupted system calls and do not stop on child process termination
    sigaction(SIGCHLD, &sa, NULL);
    signal(SIGINT, handle_sigint);



    server_fd = socket(AF_INET, SOCK_STREAM, 0);
    if (server_fd == 0) {
        perror("socket failed");
        exit(EXIT_FAILURE);
    }

    address.sin_family = AF_INET;
    address.sin_addr.s_addr = INADDR_ANY;
    address.sin_port = htons(PORT);

    if (bind(server_fd, (struct sockaddr *)&address, sizeof(address)) < 0) {
        perror("bind failed");
        exit(EXIT_FAILURE);
    }

    listen(server_fd, 5);
    printf("Server listening on port %d...\n", PORT);

    while (1) {
        new_socket = accept(server_fd, (struct sockaddr *)&address, &addrlen);
        if (new_socket < 0) {
            perror("accept");
            continue;
        }

        //pid_t pid = fork();
        if (1) {
            close(server_fd); 

            sqlite3 *db;
            if (sqlite3_open(DB_FILE, &db)) {
                const char *err = sqlite3_errmsg(db);
                write(new_socket, err, strlen(err));
                close(new_socket);
                exit(1);
            }

            handle_client(new_socket, db);
        } else if (0) {
            close(new_socket); 
        } else {
            perror("fork failed");
            close(new_socket);
        }
    }

    return 0;
}
