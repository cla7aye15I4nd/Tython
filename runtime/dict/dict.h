#ifndef TYTHON_DATASTRUCTURES_DICT_H
#define TYTHON_DATASTRUCTURES_DICT_H

#include <stdint.h>
#include "../builtins/common.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    int64_t len;
    int64_t capacity;
    int64_t* keys;
    int64_t* values;
} TythonDict;

TythonDict* TYTHON_FN(dict_empty)(void);
int64_t TYTHON_FN(dict_len)(TythonDict* d);
int64_t TYTHON_FN(dict_contains)(TythonDict* d, int64_t key);
int64_t TYTHON_FN(dict_get)(TythonDict* d, int64_t key);
void TYTHON_FN(dict_set)(TythonDict* d, int64_t key, int64_t value);
void TYTHON_FN(dict_clear)(TythonDict* d);
int64_t TYTHON_FN(dict_pop)(TythonDict* d, int64_t key);
int64_t TYTHON_FN(dict_eq)(TythonDict* a, TythonDict* b);
TythonDict* TYTHON_FN(dict_copy)(TythonDict* d);

#ifdef __cplusplus
}
#endif

#endif /* TYTHON_DATASTRUCTURES_DICT_H */

