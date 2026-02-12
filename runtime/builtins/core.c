#include "tython.h"

void TYTHON_BUILTIN(assert)(int64_t condition) {
    if (!condition) {
        fprintf(stderr, "AssertionError\n");
        exit(1);
    }
}

void* TYTHON_BUILTIN(malloc)(int64_t size) {
    void* ptr = malloc((size_t)size);
    if (!ptr) {
        fprintf(stderr, "MemoryError: allocation failed\n");
        exit(1);
    }
    return ptr;
}
