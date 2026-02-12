#ifndef TYTHON_DATASTRUCTURES_LIST_H
#define TYTHON_DATASTRUCTURES_LIST_H

#include <stdint.h>
#include "../builtins/common.h"

typedef struct {
    int64_t len;
    int64_t capacity;
    int64_t* data; /* 8-byte slots: int64_t, double (bitcast), or ptr */
} TythonList;

TythonList* TYTHON_FN(list_new)(const int64_t* data, int64_t len);
TythonList* TYTHON_FN(list_empty)(void);
int64_t TYTHON_FN(list_len)(TythonList* lst);
int64_t TYTHON_FN(list_get)(TythonList* lst, int64_t index);
void TYTHON_FN(list_set)(TythonList* lst, int64_t index, int64_t value);
void TYTHON_FN(list_append)(TythonList* lst, int64_t value);
int64_t TYTHON_FN(list_pop)(TythonList* lst);
void TYTHON_FN(list_clear)(TythonList* lst);
void TYTHON_FN(print_list_int)(TythonList* lst);
void TYTHON_FN(print_list_float)(TythonList* lst);
void TYTHON_FN(print_list_bool)(TythonList* lst);
void TYTHON_FN(print_list_str)(TythonList* lst);
void TYTHON_FN(print_list_bytes)(TythonList* lst);
void TYTHON_FN(print_list_bytearray)(TythonList* lst);
int64_t TYTHON_FN(list_contains)(TythonList* lst, int64_t value);
void TYTHON_FN(list_insert)(TythonList* lst, int64_t index, int64_t value);
void TYTHON_FN(list_remove)(TythonList* lst, int64_t value);
int64_t TYTHON_FN(list_index)(TythonList* lst, int64_t value);
int64_t TYTHON_FN(list_count)(TythonList* lst, int64_t value);
void TYTHON_FN(list_reverse)(TythonList* lst);
void TYTHON_FN(list_sort_int)(TythonList* lst);
void TYTHON_FN(list_sort_float)(TythonList* lst);
TythonList* TYTHON_FN(sorted_int)(TythonList* lst);
TythonList* TYTHON_FN(sorted_float)(TythonList* lst);
void TYTHON_FN(list_extend)(TythonList* lst, TythonList* other);
TythonList* TYTHON_FN(list_copy)(TythonList* lst);
int64_t TYTHON_FN(sum_int)(TythonList* lst);
double TYTHON_FN(sum_float)(TythonList* lst);
int64_t TYTHON_FN(sum_int_start)(TythonList* lst, int64_t start);
double TYTHON_FN(sum_float_start)(TythonList* lst, double start);
int64_t TYTHON_FN(all_list)(TythonList* lst);
int64_t TYTHON_FN(any_list)(TythonList* lst);
int64_t TYTHON_FN(list_eq_shallow)(TythonList* a, TythonList* b);
int64_t TYTHON_FN(list_eq_deep)(TythonList* a, TythonList* b, int64_t depth);

#endif /* TYTHON_DATASTRUCTURES_LIST_H */
