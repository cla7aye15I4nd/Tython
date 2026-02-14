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
TythonStr* TYTHON_FN(str_from_bytearray)(TythonByteArray* ba);
void TYTHON_FN(bytearray_append)(TythonByteArray* ba, int64_t byte_val);
void TYTHON_FN(bytearray_extend)(TythonByteArray* ba, TythonBytes* other);
void TYTHON_FN(bytearray_clear)(TythonByteArray* ba);
void TYTHON_FN(bytearray_insert)(TythonByteArray* ba, int64_t index, int64_t byte_val);
void TYTHON_FN(bytearray_remove)(TythonByteArray* ba, int64_t byte_val);
void TYTHON_FN(bytearray_reverse)(TythonByteArray* ba);
TythonByteArray* TYTHON_FN(bytearray_copy)(TythonByteArray* ba);
int64_t TYTHON_FN(bytearray_pop)(TythonByteArray* ba);
TythonByteArray* TYTHON_FN(bytearray_capitalize)(TythonByteArray* ba);
TythonByteArray* TYTHON_FN(bytearray_center)(TythonByteArray* ba, int64_t width, TythonBytes* fill);
int64_t TYTHON_FN(bytearray_count)(TythonByteArray* ba, TythonBytes* sub);
TythonStr* TYTHON_FN(bytearray_decode)(TythonByteArray* ba);
int64_t TYTHON_FN(bytearray_endswith)(TythonByteArray* ba, TythonBytes* suffix);
TythonByteArray* TYTHON_FN(bytearray_expandtabs)(TythonByteArray* ba, int64_t tabsize);
int64_t TYTHON_FN(bytearray_find)(TythonByteArray* ba, TythonBytes* sub);
TythonByteArray* TYTHON_FN(bytearray_fromhex)(TythonByteArray* self, TythonStr* hex);
TythonStr* TYTHON_FN(bytearray_hex)(TythonByteArray* ba);
int64_t TYTHON_FN(bytearray_index)(TythonByteArray* ba, TythonBytes* sub);
int64_t TYTHON_FN(bytearray_isalnum)(TythonByteArray* ba);
int64_t TYTHON_FN(bytearray_isalpha)(TythonByteArray* ba);
int64_t TYTHON_FN(bytearray_isascii)(TythonByteArray* ba);
int64_t TYTHON_FN(bytearray_isdigit)(TythonByteArray* ba);
int64_t TYTHON_FN(bytearray_islower)(TythonByteArray* ba);
int64_t TYTHON_FN(bytearray_isspace)(TythonByteArray* ba);
int64_t TYTHON_FN(bytearray_istitle)(TythonByteArray* ba);
int64_t TYTHON_FN(bytearray_isupper)(TythonByteArray* ba);
TythonByteArray* TYTHON_FN(bytearray_join)(TythonByteArray* sep, void* parts);
TythonByteArray* TYTHON_FN(bytearray_ljust)(TythonByteArray* ba, int64_t width, TythonBytes* fill);
TythonByteArray* TYTHON_FN(bytearray_lower)(TythonByteArray* ba);
TythonByteArray* TYTHON_FN(bytearray_lstrip)(TythonByteArray* ba, TythonBytes* chars);
TythonBytes* TYTHON_FN(bytearray_maketrans)(TythonByteArray* self, TythonBytes* from, TythonBytes* to);
void* TYTHON_FN(bytearray_partition)(TythonByteArray* ba, TythonBytes* sep);
TythonByteArray* TYTHON_FN(bytearray_removeprefix)(TythonByteArray* ba, TythonBytes* prefix);
TythonByteArray* TYTHON_FN(bytearray_removesuffix)(TythonByteArray* ba, TythonBytes* suffix);
TythonByteArray* TYTHON_FN(bytearray_replace)(TythonByteArray* ba, TythonBytes* old_sub, TythonBytes* new_sub);
int64_t TYTHON_FN(bytearray_rfind)(TythonByteArray* ba, TythonBytes* sub);
int64_t TYTHON_FN(bytearray_rindex)(TythonByteArray* ba, TythonBytes* sub);
TythonByteArray* TYTHON_FN(bytearray_rjust)(TythonByteArray* ba, int64_t width, TythonBytes* fill);
void* TYTHON_FN(bytearray_rpartition)(TythonByteArray* ba, TythonBytes* sep);
void* TYTHON_FN(bytearray_rsplit)(TythonByteArray* ba, TythonBytes* sep);
TythonByteArray* TYTHON_FN(bytearray_rstrip)(TythonByteArray* ba, TythonBytes* chars);
void* TYTHON_FN(bytearray_split)(TythonByteArray* ba, TythonBytes* sep);
void* TYTHON_FN(bytearray_splitlines)(TythonByteArray* ba);
int64_t TYTHON_FN(bytearray_startswith)(TythonByteArray* ba, TythonBytes* prefix);
TythonByteArray* TYTHON_FN(bytearray_strip)(TythonByteArray* ba, TythonBytes* chars);
TythonByteArray* TYTHON_FN(bytearray_swapcase)(TythonByteArray* ba);
TythonByteArray* TYTHON_FN(bytearray_title)(TythonByteArray* ba);
TythonByteArray* TYTHON_FN(bytearray_translate)(TythonByteArray* ba, TythonBytes* table);
TythonByteArray* TYTHON_FN(bytearray_upper)(TythonByteArray* ba);
TythonByteArray* TYTHON_FN(bytearray_zfill)(TythonByteArray* ba, int64_t width);
int64_t TYTHON_FN(bytearray_get)(TythonByteArray* ba, int64_t index);

#ifdef __cplusplus
}
#endif

#endif /* TYTHON_DATASTRUCTURES_BYTEARRAY_H */
