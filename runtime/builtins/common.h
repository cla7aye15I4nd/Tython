#ifndef TYTHON_BUILTINS_COMMON_H
#define TYTHON_BUILTINS_COMMON_H

#include <stdint.h>

#define TYTHON_SYM(name) __tython_##name
#define TYTHON_FN(name) TYTHON_SYM(name)
#define TYTHON_BUILTIN(name) TYTHON_FN(name)

#endif /* TYTHON_BUILTINS_COMMON_H */
