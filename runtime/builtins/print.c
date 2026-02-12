#include "tython.h"

void TYTHON_BUILTIN(print_int)(int64_t value) {
    printf("%lld", value);
}

void TYTHON_BUILTIN(print_float)(double value) {
    /* Match Python's float repr: whole floats print with ".0" */
    char buf[64];
    snprintf(buf, sizeof(buf), "%.12g", value);
    int has_dot = 0;
    for (int i = 0; buf[i]; i++) {
        if (buf[i] == '.' || buf[i] == 'e' || buf[i] == 'E'
            || buf[i] == 'n' || buf[i] == 'i') {
            has_dot = 1;
            break;
        }
    }
    printf("%s", buf);
    if (!has_dot) {
        printf(".0");
    }
}

void TYTHON_BUILTIN(print_bool)(int64_t value) {
    if (value) {
        printf("True");
    } else {
        printf("False");
    }
}

void TYTHON_BUILTIN(print_space)(void) {
    putchar(' ');
}

void TYTHON_BUILTIN(print_newline)(void) {
    putchar('\n');
}
