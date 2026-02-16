#ifndef TYTHON_DATASTRUCTURES_DICT_H
#define TYTHON_DATASTRUCTURES_DICT_H

#include <stdint.h>
#include "../builtins/common.h"
#include "../str/str.h"

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
int64_t TYTHON_FN(dict_contains_by_tag)(TythonDict* d, int64_t key, int64_t key_eq_tag);
int64_t TYTHON_FN(dict_get)(TythonDict* d, int64_t key);
int64_t TYTHON_FN(dict_get_by_tag)(TythonDict* d, int64_t key, int64_t key_eq_tag);
int64_t TYTHON_FN(dict_get_default_by_tag)(TythonDict* d, int64_t key, int64_t default_value, int64_t key_eq_tag);
void TYTHON_FN(dict_set)(TythonDict* d, int64_t key, int64_t value);
void TYTHON_FN(dict_set_by_tag)(TythonDict* d, int64_t key, int64_t value, int64_t key_eq_tag);
int64_t TYTHON_FN(dict_setdefault_by_tag)(TythonDict* d, int64_t key, int64_t default_value, int64_t key_eq_tag);
void TYTHON_FN(dict_del_by_tag)(TythonDict* d, int64_t key, int64_t key_eq_tag);
void TYTHON_FN(dict_clear)(TythonDict* d);
int64_t TYTHON_FN(dict_pop)(TythonDict* d, int64_t key);
int64_t TYTHON_FN(dict_pop_by_tag)(TythonDict* d, int64_t key, int64_t key_eq_tag);
int64_t TYTHON_FN(dict_pop_default_by_tag)(TythonDict* d, int64_t key, int64_t default_value, int64_t key_eq_tag);
int64_t TYTHON_FN(dict_eq)(TythonDict* a, TythonDict* b);
int64_t TYTHON_FN(dict_eq_by_tag)(TythonDict* a, TythonDict* b, int64_t key_eq_tag, int64_t value_eq_tag);
void TYTHON_FN(dict_update_by_tag)(TythonDict* dst, TythonDict* src, int64_t key_eq_tag);
TythonDict* TYTHON_FN(dict_or_by_tag)(TythonDict* a, TythonDict* b, int64_t key_eq_tag);
TythonDict* TYTHON_FN(dict_ior_by_tag)(TythonDict* a, TythonDict* b, int64_t key_eq_tag);
TythonDict* TYTHON_FN(dict_fromkeys_by_tag)(void* keys, int64_t value, int64_t key_eq_tag);
TythonDict* TYTHON_FN(dict_copy)(TythonDict* d);
void* TYTHON_FN(dict_items)(TythonDict* d);
void* TYTHON_FN(dict_popitem)(TythonDict* d);
void* TYTHON_FN(dict_keys)(TythonDict* d);
void* TYTHON_FN(dict_values)(TythonDict* d);
TythonStr* TYTHON_FN(dict_str_by_tag)(TythonDict* dict, int64_t key_str_tag, int64_t value_str_tag);

#ifdef __cplusplus
}
#endif

#endif /* TYTHON_DATASTRUCTURES_DICT_H */
