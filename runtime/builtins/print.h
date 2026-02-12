#ifndef TYTHON_BUILTINS_PRINT_H
#define TYTHON_BUILTINS_PRINT_H

#include "common.h"

#ifdef __cplusplus
extern "C" {
#endif

void TYTHON_BUILTIN(print_int)(int64_t value);
void TYTHON_BUILTIN(print_float)(double value);
void TYTHON_BUILTIN(print_bool)(int64_t value);
void TYTHON_BUILTIN(print_space)(void);
void TYTHON_BUILTIN(print_newline)(void);

#ifdef __cplusplus
}
#endif

#endif /* TYTHON_BUILTINS_PRINT_H */
