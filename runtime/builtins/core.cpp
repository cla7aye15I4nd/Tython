#include "tython.h"

#include <cstdio>
#include <cstdlib>
#include <cstring>

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

void* TYTHON_BUILTIN(open_read_all)(void* path_ptr) {
    auto* path = static_cast<TythonStr*>(path_ptr);
    if (!path) {
        TYTHON_FN(raise)(TYTHON_EXC_FILE_NOT_FOUND, TYTHON_FN(str_new)("null path", 9));
        __builtin_unreachable();
    }

    auto* c_path = reinterpret_cast<char*>(__tython_malloc(path->len + 1));
    std::memcpy(c_path, path->data, static_cast<size_t>(path->len));
    c_path[path->len] = '\0';

    std::FILE* f = std::fopen(c_path, "rb");
    if (!f) {
        TYTHON_FN(raise)(TYTHON_EXC_FILE_NOT_FOUND, TYTHON_FN(str_new)("file not found", 14));
        __builtin_unreachable();
    }

    std::fseek(f, 0, SEEK_END);
    long n = std::ftell(f);
    std::fseek(f, 0, SEEK_SET);
    if (n < 0) {
        std::fclose(f);
        TYTHON_FN(raise)(TYTHON_EXC_OS_ERROR, TYTHON_FN(str_new)("failed to read file", 19));
        __builtin_unreachable();
    }

    auto* data = reinterpret_cast<char*>(__tython_malloc(static_cast<int64_t>(n)));
    size_t got = std::fread(data, 1, static_cast<size_t>(n), f);
    std::fclose(f);
    if (got != static_cast<size_t>(n)) {
        TYTHON_FN(raise)(TYTHON_EXC_OS_ERROR, TYTHON_FN(str_new)("short read", 10));
        __builtin_unreachable();
    }

    return TYTHON_FN(str_new)(data, static_cast<int64_t>(n));
}
