#ifndef TYTHON_H
#define TYTHON_H

#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>
#include <math.h>

#include "builtins/builtins.h"
#include "str/str.h"
#include "bytes/bytes.h"
#include "bytearray/bytearray.h"
#include "list/list.h"
#include "dict/dict.h"
#include "set/set.h"

/* ── exception handling ───────────────────────────────────────────── */

#define TYTHON_EXC_NONE            0
#define TYTHON_EXC_EXCEPTION       1
#define TYTHON_EXC_STOP_ITERATION  2
#define TYTHON_EXC_VALUE_ERROR     3
#define TYTHON_EXC_TYPE_ERROR      4
#define TYTHON_EXC_KEY_ERROR       5
#define TYTHON_EXC_RUNTIME_ERROR   6
#define TYTHON_EXC_ZERO_DIVISION   7
#define TYTHON_EXC_OVERFLOW_ERROR  8
#define TYTHON_EXC_INDEX_ERROR     9
#define TYTHON_EXC_ATTRIBUTE_ERROR 10
#define TYTHON_EXC_NOT_IMPLEMENTED 11
#define TYTHON_EXC_NAME_ERROR      12
#define TYTHON_EXC_ARITHMETIC_ERROR 13
#define TYTHON_EXC_LOOKUP_ERROR     14
#define TYTHON_EXC_ASSERTION_ERROR  15
#define TYTHON_EXC_IMPORT_ERROR     16
#define TYTHON_EXC_MODULE_NOT_FOUND 17
#define TYTHON_EXC_FILE_NOT_FOUND   18
#define TYTHON_EXC_PERMISSION_ERROR 19
#define TYTHON_EXC_OS_ERROR         20

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    int64_t    type_tag;
    TythonStr* message;
} TythonException;

void    TYTHON_FN(raise)(int64_t type_tag, void* message);
int64_t TYTHON_FN(caught_type_tag)(void* caught_ptr);
void*   TYTHON_FN(caught_message)(void* caught_ptr);
int64_t TYTHON_FN(caught_matches)(void* caught_ptr, int64_t type_tag);
void    TYTHON_FN(print_unhandled)(int64_t type_tag, void* message);
int64_t TYTHON_FN(intrinsic_eq)(int64_t tag, int64_t lhs, int64_t rhs);
int64_t TYTHON_FN(intrinsic_lt)(int64_t tag, int64_t lhs, int64_t rhs);
int64_t TYTHON_FN(intrinsic_hash)(int64_t tag, int64_t value);

#ifdef __cplusplus
}
#endif

#endif /* TYTHON_H */
