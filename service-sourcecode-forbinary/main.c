#include <stdio.h>
#include <stdlib.h>

void function() {
    printf("This is a function.\n");
    exit(0);

}

int main(int argc, char *argv[]) {

    char buffer[64];
    strcpy(buffer, argv[1]);

    return 0;
}