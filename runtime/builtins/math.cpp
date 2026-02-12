#include "tython.h"

#include <cmath>
#include <algorithm>

int64_t TYTHON_BUILTIN(pow_int)(int64_t base, int64_t exp) {
    if (exp < 0) return 0;
    int64_t result = 1;
    while (exp > 0) {
        if (exp & 1) result *= base;
        base *= base;
        exp >>= 1;
    }
    return result;
}

int64_t TYTHON_BUILTIN(abs_int)(int64_t x)  { return x < 0 ? -x : x; }
double  TYTHON_BUILTIN(abs_float)(double x)  { return std::fabs(x); }

int64_t TYTHON_BUILTIN(min_int)(int64_t a, int64_t b) { return std::min(a, b); }
double  TYTHON_BUILTIN(min_float)(double a, double b)  { return std::min(a, b); }
int64_t TYTHON_BUILTIN(max_int)(int64_t a, int64_t b) { return std::max(a, b); }
double  TYTHON_BUILTIN(max_float)(double a, double b)  { return std::max(a, b); }

int64_t TYTHON_BUILTIN(round_float)(double x) {
    return static_cast<int64_t>(std::round(x));
}
