#ifndef TYTHON_DATASTRUCTURES_SET_H
#define TYTHON_DATASTRUCTURES_SET_H

#include <stdint.h>
#include "../builtins/common.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    int64_t len;
    int64_t capacity;
    int64_t* data;
} TythonSet;

TythonSet* TYTHON_FN(set_empty)(void);
int64_t TYTHON_FN(set_len)(TythonSet* s);
int64_t TYTHON_FN(set_contains)(TythonSet* s, int64_t value);
void TYTHON_FN(set_add)(TythonSet* s, int64_t value);
void TYTHON_FN(set_remove)(TythonSet* s, int64_t value);
void TYTHON_FN(set_discard)(TythonSet* s, int64_t value);
int64_t TYTHON_FN(set_pop)(TythonSet* s);
void TYTHON_FN(set_clear)(TythonSet* s);
int64_t TYTHON_FN(set_eq)(TythonSet* a, TythonSet* b);
TythonSet* TYTHON_FN(set_copy)(TythonSet* s);

#ifdef __cplusplus
}
#endif

#endif /* TYTHON_DATASTRUCTURES_SET_H */

