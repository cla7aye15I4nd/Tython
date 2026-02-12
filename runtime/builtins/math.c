#include "tython.h"

int64_t TYTHON_BUILTIN(pow_int)(int64_t base, int64_t exp) {
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

int64_t TYTHON_BUILTIN(abs_int)(int64_t x) {
    return x < 0 ? -x : x;
}

double TYTHON_BUILTIN(abs_float)(double x) {
    return fabs(x);
}

#define DEFINE_MINMAX(name, type, op) \
    type TYTHON_BUILTIN(name)(type a, type b) { return (a op b) ? a : b; }

DEFINE_MINMAX(min_int, int64_t, <)
DEFINE_MINMAX(min_float, double, <)
DEFINE_MINMAX(max_int, int64_t, >)
DEFINE_MINMAX(max_float, double, >)

#undef DEFINE_MINMAX

int64_t TYTHON_BUILTIN(round_float)(double x) {
    return (int64_t)round(x);
}
