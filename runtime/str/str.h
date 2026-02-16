#ifndef TYTHON_DATASTRUCTURES_STR_H
#define TYTHON_DATASTRUCTURES_STR_H

#include <stdint.h>
#include "../builtins/common.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    int64_t len;
    char data[]; /* flexible array — data stored inline after len */
} TythonStr;

TythonStr* TYTHON_FN(str_new)(const char* data, int64_t len);
TythonStr* TYTHON_FN(str_concat)(TythonStr* a, TythonStr* b);
TythonStr* TYTHON_FN(str_repeat)(TythonStr* s, int64_t n);
int64_t TYTHON_FN(str_len)(TythonStr* s);
TythonStr* TYTHON_FN(str_get_char)(TythonStr* s, int64_t index);
int64_t TYTHON_FN(str_cmp)(TythonStr* a, TythonStr* b);
int64_t TYTHON_FN(str_eq)(TythonStr* a, TythonStr* b);
void TYTHON_FN(print_str)(TythonStr* s);
TythonStr* TYTHON_FN(str_from_int)(int64_t v);
TythonStr* TYTHON_FN(str_from_float)(double v);
TythonStr* TYTHON_FN(str_from_bool)(int64_t v);
TythonStr* TYTHON_FN(str_format_int)(int64_t v, TythonStr* spec);
TythonStr* TYTHON_FN(str_format_float)(double v, TythonStr* spec);
int64_t TYTHON_FN(str_contains)(TythonStr* haystack, TythonStr* needle);
TythonStr* TYTHON_FN(repr_str)(TythonStr* s);
TythonStr* TYTHON_FN(str_read)(TythonStr* s);
TythonStr* TYTHON_FN(str_strip)(TythonStr* s);
void* TYTHON_FN(str_split)(TythonStr* s, TythonStr* sep);
TythonStr* TYTHON_FN(str_join)(TythonStr* sep, void* parts);
int64_t TYTHON_FN(str_hash)(TythonStr* s);
void* TYTHON_FN(set_from_str)(TythonStr* s);

#ifdef __cplusplus
}
#endif

#endif /* TYTHON_DATASTRUCTURES_STR_H */
