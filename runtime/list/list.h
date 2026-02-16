#ifndef TYTHON_DATASTRUCTURES_LIST_H
#define TYTHON_DATASTRUCTURES_LIST_H

#include <stdint.h>
#include "../builtins/common.h"
#include "../str/str.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    int64_t len;
    int64_t capacity;
    int64_t* data; /* 8-byte slots: int64_t, double (bitcast), or ptr */
} TythonList;

TythonList* TYTHON_FN(list_new)(const int64_t* data, int64_t len);
TythonList* TYTHON_FN(list_empty)(void);
TythonList* TYTHON_FN(list_concat)(TythonList* a, TythonList* b);
int64_t TYTHON_FN(list_len)(TythonList* lst);
int64_t TYTHON_FN(list_get)(TythonList* lst, int64_t index);
TythonList* TYTHON_FN(list_slice)(TythonList* lst, int64_t start, int64_t stop);
TythonList* TYTHON_FN(list_repeat)(TythonList* lst, int64_t n);
void TYTHON_FN(list_set)(TythonList* lst, int64_t index, int64_t value);
void TYTHON_FN(list_append)(TythonList* lst, int64_t value);
int64_t TYTHON_FN(list_pop)(TythonList* lst);
void TYTHON_FN(list_clear)(TythonList* lst);
int64_t TYTHON_FN(list_contains)(TythonList* lst, int64_t value);
void TYTHON_FN(list_insert)(TythonList* lst, int64_t index, int64_t value);
void TYTHON_FN(list_remove)(TythonList* lst, int64_t value);
int64_t TYTHON_FN(list_index)(TythonList* lst, int64_t value);
int64_t TYTHON_FN(list_count)(TythonList* lst, int64_t value);
void TYTHON_FN(list_reverse)(TythonList* lst);
void TYTHON_FN(list_sort_int)(TythonList* lst);
void TYTHON_FN(list_sort_float)(TythonList* lst);
void TYTHON_FN(list_sort_str)(TythonList* lst);
void TYTHON_FN(list_sort_bytes)(TythonList* lst);
void TYTHON_FN(list_sort_bytearray)(TythonList* lst);
TythonList* TYTHON_FN(sorted_int)(TythonList* lst);
TythonList* TYTHON_FN(sorted_float)(TythonList* lst);
TythonList* TYTHON_FN(sorted_str)(TythonList* lst);
TythonList* TYTHON_FN(sorted_bytes)(TythonList* lst);
TythonList* TYTHON_FN(sorted_bytearray)(TythonList* lst);
TythonList* TYTHON_FN(reversed_list)(TythonList* lst);
void TYTHON_FN(list_extend)(TythonList* lst, TythonList* other);
TythonList* TYTHON_FN(list_copy)(TythonList* lst);
TythonList* TYTHON_FN(list_iadd)(TythonList* lst, TythonList* other);
TythonList* TYTHON_FN(list_imul)(TythonList* lst, int64_t n);
void TYTHON_FN(list_del)(TythonList* lst, int64_t index);
TythonList* TYTHON_FN(range_1)(int64_t stop);
TythonList* TYTHON_FN(range_2)(int64_t start, int64_t stop);
TythonList* TYTHON_FN(range_3)(int64_t start, int64_t stop, int64_t step);
int64_t TYTHON_FN(sum_int)(TythonList* lst);
double TYTHON_FN(sum_float)(TythonList* lst);
int64_t TYTHON_FN(sum_int_start)(TythonList* lst, int64_t start);
double TYTHON_FN(sum_float_start)(TythonList* lst, double start);
int64_t TYTHON_FN(all_list)(TythonList* lst);
int64_t TYTHON_FN(any_list)(TythonList* lst);
int64_t TYTHON_FN(max_list_int)(TythonList* lst);
double TYTHON_FN(max_list_float)(TythonList* lst);
int64_t TYTHON_FN(list_eq_shallow)(TythonList* a, TythonList* b);
int64_t TYTHON_FN(list_eq_deep)(TythonList* a, TythonList* b, int64_t depth);
int64_t TYTHON_FN(list_eq_by_tag)(TythonList* a, TythonList* b, int64_t eq_tag);
int64_t TYTHON_FN(list_lt_by_tag)(TythonList* a, TythonList* b, int64_t lt_tag);
int64_t TYTHON_FN(list_contains_by_tag)(TythonList* lst, int64_t value, int64_t eq_tag);
int64_t TYTHON_FN(list_index_by_tag)(TythonList* lst, int64_t value, int64_t eq_tag);
int64_t TYTHON_FN(list_count_by_tag)(TythonList* lst, int64_t value, int64_t eq_tag);
void TYTHON_FN(list_remove_by_tag)(TythonList* lst, int64_t value, int64_t eq_tag);
void TYTHON_FN(list_sort_by_tag)(TythonList* lst, int64_t lt_tag);
TythonList* TYTHON_FN(sorted_by_tag)(TythonList* lst, int64_t lt_tag);
TythonStr* TYTHON_FN(list_str_by_tag)(TythonList* list, int64_t elem_str_tag);

#ifdef __cplusplus
}
#endif

#endif /* TYTHON_DATASTRUCTURES_LIST_H */
