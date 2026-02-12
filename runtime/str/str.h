#ifndef TYTHON_DATASTRUCTURES_STR_H
#define TYTHON_DATASTRUCTURES_STR_H

#include <stdint.h>
#include "../builtins/common.h"

typedef struct {
    int64_t len;
    char* data;
} TythonStr;

TythonStr* TYTHON_FN(str_new)(const char* data, int64_t len);
TythonStr* TYTHON_FN(str_concat)(TythonStr* a, TythonStr* b);
TythonStr* TYTHON_FN(str_repeat)(TythonStr* s, int64_t n);
int64_t TYTHON_FN(str_len)(TythonStr* s);
int64_t TYTHON_FN(str_cmp)(TythonStr* a, TythonStr* b);
int64_t TYTHON_FN(str_eq)(TythonStr* a, TythonStr* b);
void TYTHON_FN(print_str)(TythonStr* s);
TythonStr* TYTHON_FN(str_from_int)(int64_t v);
TythonStr* TYTHON_FN(str_from_float)(double v);
TythonStr* TYTHON_FN(str_from_bool)(int64_t v);
int64_t TYTHON_FN(str_contains)(TythonStr* haystack, TythonStr* needle);

#endif /* TYTHON_DATASTRUCTURES_STR_H */
