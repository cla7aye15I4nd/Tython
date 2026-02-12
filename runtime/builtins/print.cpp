#include "tython.h"

#include <cstdio>

void TYTHON_BUILTIN(print_int)(int64_t value) {
    std::printf("%lld", (long long)value);
}

void TYTHON_BUILTIN(print_float)(double value) {
    char buf[64];
    std::snprintf(buf, sizeof(buf), "%.12g", value);
    bool has_dot = false;
    for (int i = 0; buf[i]; i++) {
        if (buf[i] == '.' || buf[i] == 'e' || buf[i] == 'E'
            || buf[i] == 'n' || buf[i] == 'i') {
            has_dot = true;
            break;
        }
    }
    std::printf("%s", buf);
    if (!has_dot) std::printf(".0");
}

void TYTHON_BUILTIN(print_bool)(int64_t value) {
    std::printf("%s", value ? "True" : "False");
}

void TYTHON_BUILTIN(print_space)(void) { std::putchar(' '); }

void TYTHON_BUILTIN(print_newline)(void) { std::putchar('\n'); }
