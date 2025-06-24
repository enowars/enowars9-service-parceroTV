#include <stdio.h>
#include <stdlib.h>
#include <string.h>

void function() {
    printf("This is a function.\n");
    exit(0);
}

void vulnerable() {
    char buffer[256];
    gets(buffer); 
}

int main() {
    vulnerable();
    return 0;
}
