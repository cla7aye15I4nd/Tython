#ifndef TYTHON_DATASTRUCTURES_SET_H
#define TYTHON_DATASTRUCTURES_SET_H

#include <stdint.h>
#include "../builtins/common.h"
#include "../str/str.h"

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
int64_t TYTHON_FN(set_contains_by_tag)(TythonSet* s, int64_t value, int64_t eq_ops_handle);
void TYTHON_FN(set_add)(TythonSet* s, int64_t value);
void TYTHON_FN(set_add_by_tag)(TythonSet* s, int64_t value, int64_t eq_ops_handle);
void TYTHON_FN(set_remove)(TythonSet* s, int64_t value);
void TYTHON_FN(set_remove_by_tag)(TythonSet* s, int64_t value, int64_t eq_ops_handle);
void TYTHON_FN(set_discard)(TythonSet* s, int64_t value);
void TYTHON_FN(set_discard_by_tag)(TythonSet* s, int64_t value, int64_t eq_ops_handle);
TythonSet* TYTHON_FN(set_union_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_ops_handle);
void TYTHON_FN(set_update_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_ops_handle);
TythonSet* TYTHON_FN(set_intersection_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_ops_handle);
void TYTHON_FN(set_intersection_update_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_ops_handle);
TythonSet* TYTHON_FN(set_difference_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_ops_handle);
void TYTHON_FN(set_difference_update_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_ops_handle);
TythonSet* TYTHON_FN(set_symmetric_difference_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_ops_handle);
void TYTHON_FN(set_symmetric_difference_update_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_ops_handle);
int64_t TYTHON_FN(set_isdisjoint_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_ops_handle);
int64_t TYTHON_FN(set_issubset_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_ops_handle);
int64_t TYTHON_FN(set_issuperset_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_ops_handle);
int64_t TYTHON_FN(set_lt_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_ops_handle);
int64_t TYTHON_FN(set_le_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_ops_handle);
int64_t TYTHON_FN(set_gt_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_ops_handle);
int64_t TYTHON_FN(set_ge_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_ops_handle);
TythonSet* TYTHON_FN(set_iand_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_ops_handle);
TythonSet* TYTHON_FN(set_ior_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_ops_handle);
TythonSet* TYTHON_FN(set_isub_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_ops_handle);
TythonSet* TYTHON_FN(set_ixor_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_ops_handle);
int64_t TYTHON_FN(set_pop)(TythonSet* s);
void TYTHON_FN(set_clear)(TythonSet* s);
int64_t TYTHON_FN(set_eq)(TythonSet* a, TythonSet* b);
int64_t TYTHON_FN(set_eq_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_ops_handle);
TythonSet* TYTHON_FN(set_copy)(TythonSet* s);
TythonStr* TYTHON_FN(set_str_by_tag)(TythonSet* set, int64_t elem_str_ops_handle);

#ifdef __cplusplus
}
#endif

#endif /* TYTHON_DATASTRUCTURES_SET_H */
