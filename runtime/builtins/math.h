#ifndef TYTHON_BUILTINS_MATH_H
#define TYTHON_BUILTINS_MATH_H

#include "common.h"

#ifdef __cplusplus
extern "C" {
#endif

int64_t TYTHON_BUILTIN(pow_int)(int64_t base, int64_t exp);
int64_t TYTHON_BUILTIN(abs_int)(int64_t x);
double TYTHON_BUILTIN(abs_float)(double x);
int64_t TYTHON_BUILTIN(min_int)(int64_t a, int64_t b);
double TYTHON_BUILTIN(min_float)(double a, double b);
int64_t TYTHON_BUILTIN(max_int)(int64_t a, int64_t b);
double TYTHON_BUILTIN(max_float)(double a, double b);
int64_t TYTHON_BUILTIN(round_float)(double x);
double TYTHON_BUILTIN(math_log)(double x);
double TYTHON_BUILTIN(math_exp)(double x);
void TYTHON_BUILTIN(random_seed)(int64_t seed);
double TYTHON_BUILTIN(random_gauss)(double mu, double sigma);
void TYTHON_BUILTIN(random_shuffle)(void* lst);
void* TYTHON_BUILTIN(random_choices_int)(void* population, void* weights);

#ifdef __cplusplus
}
#endif

#endif /* TYTHON_BUILTINS_MATH_H */
