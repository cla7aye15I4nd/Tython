#include "tython.h"

#include <cstdio>
#include <cstdlib>

void TYTHON_BUILTIN(assert)(int64_t condition) {
    if (!condition) {
        std::fprintf(stderr, "AssertionError\n");
        std::exit(1);
    }
}

void* TYTHON_BUILTIN(malloc)(int64_t size) {
    void* ptr = std::malloc(static_cast<size_t>(size));
    if (!ptr) {
        std::fprintf(stderr, "MemoryError: allocation failed\n");
        std::exit(1);
    }
    return ptr;
}
