#include "tython.h"

#include <cmath>
#include <algorithm>
#include <cstdint>
#include <cstring>
#include <random>

static std::mt19937_64 g_rng(0);

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

double TYTHON_BUILTIN(math_log)(double x) {
    return std::log(x);
}

double TYTHON_BUILTIN(math_exp)(double x) {
    return std::exp(x);
}

void TYTHON_BUILTIN(random_seed)(int64_t seed) {
    g_rng.seed(static_cast<uint64_t>(seed));
}

double TYTHON_BUILTIN(random_gauss)(double mu, double sigma) {
    std::normal_distribution<double> dist(mu, sigma);
    return dist(g_rng);
}

void TYTHON_BUILTIN(random_shuffle)(void* lst) {
    auto* list = static_cast<TythonList*>(lst);
    if (!list || list->len <= 1) {
        return;
    }
    for (int64_t i = list->len - 1; i > 0; --i) {
        std::uniform_int_distribution<int64_t> dist(0, i);
        int64_t j = dist(g_rng);
        std::swap(list->data[i], list->data[j]);
    }
}

void* TYTHON_BUILTIN(random_choices_int)(void* population, void* weights) {
    auto* pop = static_cast<TythonList*>(population);
    auto* w = static_cast<TythonList*>(weights);
    if (!pop || !w || pop->len != w->len || pop->len <= 0) {
        TYTHON_FN(raise)(TYTHON_EXC_VALUE_ERROR, TYTHON_FN(str_new)("invalid population/weights", 26));
        __builtin_unreachable();
    }

    double total = 0.0;
    for (int64_t i = 0; i < w->len; ++i) {
        double wi;
        std::memcpy(&wi, &w->data[i], sizeof(double));
        if (wi < 0.0) {
            TYTHON_FN(raise)(TYTHON_EXC_VALUE_ERROR, TYTHON_FN(str_new)("weights must be non-negative", 28));
            __builtin_unreachable();
        }
        total += wi;
    }
    if (total <= 0.0) {
        TYTHON_FN(raise)(TYTHON_EXC_VALUE_ERROR, TYTHON_FN(str_new)("total weight must be positive", 29));
        __builtin_unreachable();
    }

    std::uniform_real_distribution<double> dist(0.0, total);
    double r = dist(g_rng);
    double acc = 0.0;
    int64_t picked = pop->data[pop->len - 1];
    for (int64_t i = 0; i < w->len; ++i) {
        double wi;
        std::memcpy(&wi, &w->data[i], sizeof(double));
        acc += wi;
        if (r <= acc) {
            picked = pop->data[i];
            break;
        }
    }

    return TYTHON_FN(list_new)(&picked, 1);
}
