#ifndef TYTHON_BUILTINS_CORE_H
#define TYTHON_BUILTINS_CORE_H

#include "common.h"

#ifdef __cplusplus
extern "C" {
#endif

void TYTHON_BUILTIN(assert)(int64_t condition);
void* TYTHON_BUILTIN(malloc)(int64_t size);
void* TYTHON_BUILTIN(open_read_all)(void* path);

#ifdef __cplusplus
}
#endif

#endif /* TYTHON_BUILTINS_CORE_H */
