#ifndef TYTHON_BUILTINS_CORE_H
#define TYTHON_BUILTINS_CORE_H

#include "common.h"

void TYTHON_BUILTIN(assert)(int64_t condition);
void* TYTHON_BUILTIN(malloc)(int64_t size);

#endif /* TYTHON_BUILTINS_CORE_H */
