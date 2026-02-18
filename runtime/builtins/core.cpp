#include "tython.h"
#include "gc/gc.h"

#include <cerrno>
#include <cstdio>
#include <cstdlib>
#include <cstring>

namespace {

struct TythonFile {
    std::FILE* fp;
    int64_t can_read;
    int64_t can_write;
};

[[noreturn]] void raise_value_error(const char* msg, int64_t len) {
    TYTHON_FN(raise)(TYTHON_EXC_VALUE_ERROR, TYTHON_FN(str_new)(msg, len));
    __builtin_unreachable();
}

[[noreturn]] void raise_open_error() {
    if (errno == ENOENT) {
        TYTHON_FN(raise)(TYTHON_EXC_FILE_NOT_FOUND, TYTHON_FN(str_new)("file not found", 14));
    } else if (errno == EACCES) {
        TYTHON_FN(raise)(TYTHON_EXC_PERMISSION_ERROR, TYTHON_FN(str_new)("permission denied", 17));
    } else {
        TYTHON_FN(raise)(TYTHON_EXC_OS_ERROR, TYTHON_FN(str_new)("failed to open file", 19));
    }
    __builtin_unreachable();
}

[[noreturn]] void raise_os_error(const char* msg, int64_t len) {
    TYTHON_FN(raise)(TYTHON_EXC_OS_ERROR, TYTHON_FN(str_new)(msg, len));
    __builtin_unreachable();
}

char* str_to_c_string(TythonStr* s) {
    auto* out = reinterpret_cast<char*>(__tython_malloc(s->len + 1));
    std::memcpy(out, s->data, static_cast<size_t>(s->len));
    out[s->len] = '\0';
    return out;
}

bool decode_mode(
    TythonStr* mode,
    const char** fopen_mode,
    int64_t* can_read,
    int64_t* can_write
) {
    const int64_t len = mode->len;
    const char* data = mode->data;
    if (len == 1) {
        if (data[0] == 'r') {
            *fopen_mode = "rb";
            *can_read = 1;
            *can_write = 0;
            return true;
        }
        if (data[0] == 'w') {
            *fopen_mode = "wb";
            *can_read = 0;
            *can_write = 1;
            return true;
        }
        if (data[0] == 'a') {
            *fopen_mode = "ab";
            *can_read = 0;
            *can_write = 1;
            return true;
        }
        return false;
    }

    if (len == 2 && data[1] == 'b') {
        if (data[0] == 'r') {
            *fopen_mode = "rb";
            *can_read = 1;
            *can_write = 0;
            return true;
        }
        if (data[0] == 'w') {
            *fopen_mode = "wb";
            *can_read = 0;
            *can_write = 1;
            return true;
        }
        if (data[0] == 'a') {
            *fopen_mode = "ab";
            *can_read = 0;
            *can_write = 1;
            return true;
        }
    }

    return false;
}

TythonFile* require_open_file(void* file_ptr) {
    auto* file = static_cast<TythonFile*>(file_ptr);
    if (!file || !file->fp) {
        raise_value_error("I/O operation on closed file", 28);
    }
    return file;
}

} // namespace

void TYTHON_BUILTIN(assert)(int64_t condition) {
    if (!condition) {
        std::fprintf(stderr, "AssertionError\n");
        std::exit(1);
    }
}

void* TYTHON_BUILTIN(malloc)(int64_t size) {
    return __tython_gc_malloc(size);
}

void* TYTHON_BUILTIN(open)(void* path_ptr, void* mode_ptr) {
    auto* path = static_cast<TythonStr*>(path_ptr);
    auto* mode = static_cast<TythonStr*>(mode_ptr);
    if (!path || !mode) {
        raise_value_error("open() path/mode must be str", 28);
    }

    const char* fopen_mode = nullptr;
    int64_t can_read = 0;
    int64_t can_write = 0;
    if (!decode_mode(mode, &fopen_mode, &can_read, &can_write)) {
        raise_value_error("unsupported file mode", 21);
    }

    auto* c_path = str_to_c_string(path);

    std::FILE* f = std::fopen(c_path, fopen_mode);
    if (!f) {
        raise_open_error();
    }

    auto* file = reinterpret_cast<TythonFile*>(__tython_malloc(sizeof(TythonFile)));
    file->fp = f;
    file->can_read = can_read;
    file->can_write = can_write;
    return file;
}

void* TYTHON_BUILTIN(file_read)(void* file_ptr) {
    auto* file = require_open_file(file_ptr);
    if (!file->can_read) {
        raise_value_error("file not open for reading", 25);
    }

    long start = std::ftell(file->fp);
    if (start < 0) {
        raise_os_error("failed to tell file position", 28);
    }
    if (std::fseek(file->fp, 0, SEEK_END) != 0) {
        raise_os_error("failed to seek file", 19);
    }
    long end = std::ftell(file->fp);
    if (end < 0) {
        raise_os_error("failed to tell file position", 28);
    }
    if (std::fseek(file->fp, start, SEEK_SET) != 0) {
        raise_os_error("failed to seek file", 19);
    }

    long n = end - start;
    if (n < 0) {
        raise_os_error("invalid file position", 21);
    }
    if (n == 0) {
        return TYTHON_FN(str_new)("", 0);
    }

    auto* data = reinterpret_cast<char*>(__tython_malloc(static_cast<int64_t>(n)));
    size_t got = std::fread(data, 1, static_cast<size_t>(n), file->fp);
    if (got != static_cast<size_t>(n)) {
        raise_os_error("short read", 10);
    }

    return TYTHON_FN(str_new)(data, static_cast<int64_t>(n));
}

int64_t TYTHON_BUILTIN(file_write)(void* file_ptr, void* data_ptr) {
    auto* file = require_open_file(file_ptr);
    if (!file->can_write) {
        raise_value_error("file not open for writing", 25);
    }
    auto* s = static_cast<TythonStr*>(data_ptr);
    if (!s) {
        raise_value_error("write() argument must be str", 28);
    }

    size_t wrote = std::fwrite(s->data, 1, static_cast<size_t>(s->len), file->fp);
    if (wrote != static_cast<size_t>(s->len)) {
        raise_os_error("short write", 11);
    }
    return s->len;
}

void TYTHON_BUILTIN(file_close)(void* file_ptr) {
    auto* file = static_cast<TythonFile*>(file_ptr);
    if (!file || !file->fp) {
        return;
    }
    std::fclose(file->fp);
    file->fp = nullptr;
    file->can_read = 0;
    file->can_write = 0;
}
