#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>

void __tython_print_int(int64_t value) {
    printf("%lld", value);
}

void __tython_print_float(double value) {
    printf("%g", value);
}

void __tython_print_bool(int64_t value) {
    if (value) {
        printf("True");
    } else {
        printf("False");
    }
}

void __tython_print_space() {
    putchar(' ');
}

void __tython_print_newline() {
    putchar('\n');
}

void __tython_assert(int64_t condition) {
    if (!condition) {
        fprintf(stderr, "AssertionError\n");
        exit(1);
    }
}
