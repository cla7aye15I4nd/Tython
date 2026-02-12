#ifndef TYTHON_DATASTRUCTURES_BYTEARRAY_H
#define TYTHON_DATASTRUCTURES_BYTEARRAY_H

#include <stdint.h>
#include "../builtins/common.h"
#include "../bytes/bytes.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    int64_t len;
    int64_t capacity;
    uint8_t* data;
} TythonByteArray;

TythonByteArray* TYTHON_FN(bytearray_new)(const uint8_t* data, int64_t len);
TythonByteArray* TYTHON_FN(bytearray_empty)(void);
TythonByteArray* TYTHON_FN(bytearray_from_int)(int64_t n);
TythonByteArray* TYTHON_FN(bytearray_from_bytes)(TythonBytes* b);
TythonByteArray* TYTHON_FN(bytearray_concat)(TythonByteArray* a, TythonByteArray* b);
TythonByteArray* TYTHON_FN(bytearray_repeat)(TythonByteArray* s, int64_t n);
int64_t TYTHON_FN(bytearray_len)(TythonByteArray* ba);
int64_t TYTHON_FN(bytearray_cmp)(TythonByteArray* a, TythonByteArray* b);
int64_t TYTHON_FN(bytearray_eq)(TythonByteArray* a, TythonByteArray* b);
void TYTHON_FN(print_bytearray)(TythonByteArray* ba);
void TYTHON_FN(bytearray_append)(TythonByteArray* ba, int64_t byte_val);
void TYTHON_FN(bytearray_extend)(TythonByteArray* ba, TythonBytes* other);
void TYTHON_FN(bytearray_clear)(TythonByteArray* ba);
void TYTHON_FN(bytearray_insert)(TythonByteArray* ba, int64_t index, int64_t byte_val);
void TYTHON_FN(bytearray_remove)(TythonByteArray* ba, int64_t byte_val);
void TYTHON_FN(bytearray_reverse)(TythonByteArray* ba);

#ifdef __cplusplus
}
#endif

#endif /* TYTHON_DATASTRUCTURES_BYTEARRAY_H */
