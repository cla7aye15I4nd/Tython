#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <math.h>

void __tython_print_int(int64_t value) {
    printf("%lld", value);
}

void __tython_print_float(double value) {
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

int64_t __tython_pow_int(int64_t base, int64_t exp) {
    if (exp < 0) {
        return 0;
    }
    int64_t result = 1;
    while (exp > 0) {
        if (exp & 1) {
            result *= base;
        }
        base *= base;
        exp >>= 1;
    }
    return result;
}

int64_t __tython_abs_int(int64_t x) {
    return x < 0 ? -x : x;
}

double __tython_abs_float(double x) {
    return fabs(x);
}

int64_t __tython_min_int(int64_t a, int64_t b) {
    return a < b ? a : b;
}

double __tython_min_float(double a, double b) {
    return a < b ? a : b;
}

int64_t __tython_max_int(int64_t a, int64_t b) {
    return a > b ? a : b;
}

double __tython_max_float(double a, double b) {
    return a > b ? a : b;
}

int64_t __tython_round_float(double x) {
    return (int64_t)round(x);
}
