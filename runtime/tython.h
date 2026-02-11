#ifndef TYTHON_H
#define TYTHON_H

#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>
#include <math.h>


/* ── types ─────────────────────────────────────────────────────────── */

typedef struct {
    int64_t len;
    char* data;
} TythonStr;

typedef struct {
    int64_t len;
    uint8_t* data;
} TythonBytes;

typedef struct {
    int64_t len;
    int64_t capacity;
    uint8_t* data;
} TythonByteArray;

/* ── builtins ──────────────────────────────────────────────────────── */

void    __tython_print_int(int64_t value);
void    __tython_print_float(double value);
void    __tython_print_bool(int64_t value);
void    __tython_print_space(void);
void    __tython_print_newline(void);
void    __tython_assert(int64_t condition);

int64_t __tython_pow_int(int64_t base, int64_t exp);
int64_t __tython_abs_int(int64_t x);
double  __tython_abs_float(double x);
int64_t __tython_min_int(int64_t a, int64_t b);
double  __tython_min_float(double a, double b);
int64_t __tython_max_int(int64_t a, int64_t b);
double  __tython_max_float(double a, double b);
int64_t __tython_round_float(double x);

void*   __tython_malloc(int64_t size);

/* ── str ───────────────────────────────────────────────────────────── */

TythonStr* __tython_str_new(const char* data, int64_t len);
TythonStr* __tython_str_concat(TythonStr* a, TythonStr* b);
TythonStr* __tython_str_repeat(TythonStr* s, int64_t n);
int64_t    __tython_str_len(TythonStr* s);
int64_t    __tython_str_cmp(TythonStr* a, TythonStr* b);
int64_t    __tython_str_eq(TythonStr* a, TythonStr* b);
void       __tython_print_str(TythonStr* s);
TythonStr* __tython_str_from_int(int64_t v);
TythonStr* __tython_str_from_float(double v);
TythonStr* __tython_str_from_bool(int64_t v);

/* ── bytes ─────────────────────────────────────────────────────────── */

TythonBytes* __tython_bytes_new(const uint8_t* data, int64_t len);
TythonBytes* __tython_bytes_concat(TythonBytes* a, TythonBytes* b);
TythonBytes* __tython_bytes_repeat(TythonBytes* s, int64_t n);
int64_t      __tython_bytes_len(TythonBytes* b);
int64_t      __tython_bytes_cmp(TythonBytes* a, TythonBytes* b);
int64_t      __tython_bytes_eq(TythonBytes* a, TythonBytes* b);
void         __tython_print_bytes(TythonBytes* b);
TythonBytes* __tython_bytes_from_int(int64_t n);
TythonBytes* __tython_bytes_from_str(TythonStr* s);

void print_bytes_repr(const uint8_t* data, int64_t len);

/* ── bytearray ─────────────────────────────────────────────────────── */

TythonByteArray* __tython_bytearray_new(const uint8_t* data, int64_t len);
TythonByteArray* __tython_bytearray_empty(void);
TythonByteArray* __tython_bytearray_from_int(int64_t n);
TythonByteArray* __tython_bytearray_from_bytes(TythonBytes* b);
TythonByteArray* __tython_bytearray_concat(TythonByteArray* a, TythonByteArray* b);
TythonByteArray* __tython_bytearray_repeat(TythonByteArray* s, int64_t n);
int64_t          __tython_bytearray_len(TythonByteArray* ba);
int64_t          __tython_bytearray_cmp(TythonByteArray* a, TythonByteArray* b);
int64_t          __tython_bytearray_eq(TythonByteArray* a, TythonByteArray* b);
void             __tython_print_bytearray(TythonByteArray* ba);
void             __tython_bytearray_append(TythonByteArray* ba, int64_t byte_val);
void             __tython_bytearray_extend(TythonByteArray* ba, TythonBytes* other);
void             __tython_bytearray_clear(TythonByteArray* ba);

/* ── list ──────────────────────────────────────────────────────────── */

typedef struct {
    int64_t len;
    int64_t capacity;
    int64_t* data;    /* 8-byte slots: int64_t, double (bitcast), or ptr */
} TythonList;

TythonList* __tython_list_new(const int64_t* data, int64_t len);
TythonList* __tython_list_empty(void);
int64_t     __tython_list_len(TythonList* lst);
int64_t     __tython_list_get(TythonList* lst, int64_t index);
void        __tython_list_set(TythonList* lst, int64_t index, int64_t value);
void        __tython_list_append(TythonList* lst, int64_t value);
int64_t     __tython_list_pop(TythonList* lst);
void        __tython_list_clear(TythonList* lst);

void        __tython_print_list_int(TythonList* lst);
void        __tython_print_list_float(TythonList* lst);
void        __tython_print_list_bool(TythonList* lst);
void        __tython_print_list_str(TythonList* lst);
void        __tython_print_list_bytes(TythonList* lst);
void        __tython_print_list_bytearray(TythonList* lst);

int64_t     __tython_list_eq_shallow(TythonList* a, TythonList* b);
int64_t     __tython_list_eq_deep(TythonList* a, TythonList* b, int64_t depth);

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
typedef struct {
    int64_t    type_tag;
    TythonStr* message;
} TythonException;

void    __tython_raise(int64_t type_tag, void* message);
int64_t __tython_caught_type_tag(void* caught_ptr);
void*   __tython_caught_message(void* caught_ptr);
int64_t __tython_caught_matches(void* caught_ptr, int64_t type_tag);
void    __tython_print_unhandled(int64_t type_tag, void* message);

#endif /* TYTHON_H */
