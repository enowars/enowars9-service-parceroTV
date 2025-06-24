#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/wait.h>
#include "signals.h"

// Handle Ctrl+C or kill signals
void handle_sigint(int sig)
{
    printf("\nShutting down server...\n");
    exit(0);
}

// Handle child process termination
void handle_sigchld(int sig)
{
    int status;
    pid_t pid;

    //Clean up zombie processes
    while ((pid = waitpid(-1, &status, WNOHANG)) > 0)
    {
        if (WIFSIGNALED(status))
        {
            int sig = WTERMSIG(status);
            fprintf(stderr, "Child %d died with signal %d (e.g., segfault)\n", pid, sig);
        }
        else if (WIFEXITED(status))
        {
            fprintf(stderr, "Child %d exited with code %d\n", pid, WEXITSTATUS(status));
        }
    }
}
