#ifndef TYTHON_DATASTRUCTURES_BYTES_H
#define TYTHON_DATASTRUCTURES_BYTES_H

#include <stdint.h>
#include "../builtins/common.h"
#include "../str/str.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef struct {
    int64_t len;
    uint8_t data[]; /* flexible array — data stored inline after len */
} TythonBytes;

TythonBytes* TYTHON_FN(bytes_new)(const uint8_t* data, int64_t len);
TythonBytes* TYTHON_FN(bytes_concat)(TythonBytes* a, TythonBytes* b);
TythonBytes* TYTHON_FN(bytes_repeat)(TythonBytes* s, int64_t n);
int64_t TYTHON_FN(bytes_len)(TythonBytes* b);
int64_t TYTHON_FN(bytes_cmp)(TythonBytes* a, TythonBytes* b);
int64_t TYTHON_FN(bytes_eq)(TythonBytes* a, TythonBytes* b);
void TYTHON_FN(print_bytes)(TythonBytes* b);
TythonBytes* TYTHON_FN(bytes_from_int)(int64_t n);
TythonBytes* TYTHON_FN(bytes_from_str)(TythonStr* s);
TythonStr* TYTHON_FN(str_from_bytes)(TythonBytes* b);
TythonBytes* TYTHON_FN(bytes_capitalize)(TythonBytes* b);
TythonBytes* TYTHON_FN(bytes_center)(TythonBytes* b, int64_t width, TythonBytes* fill);
int64_t TYTHON_FN(bytes_count)(TythonBytes* b, TythonBytes* sub);
TythonStr* TYTHON_FN(bytes_decode)(TythonBytes* b);
int64_t TYTHON_FN(bytes_endswith)(TythonBytes* b, TythonBytes* suffix);
TythonBytes* TYTHON_FN(bytes_expandtabs)(TythonBytes* b, int64_t tabsize);
int64_t TYTHON_FN(bytes_find)(TythonBytes* b, TythonBytes* sub);
TythonBytes* TYTHON_FN(bytes_fromhex)(TythonBytes* _self, TythonStr* hex);
TythonStr* TYTHON_FN(bytes_hex)(TythonBytes* b);
int64_t TYTHON_FN(bytes_index)(TythonBytes* b, TythonBytes* sub);
int64_t TYTHON_FN(bytes_isalnum)(TythonBytes* b);
int64_t TYTHON_FN(bytes_isalpha)(TythonBytes* b);
int64_t TYTHON_FN(bytes_isascii)(TythonBytes* b);
int64_t TYTHON_FN(bytes_isdigit)(TythonBytes* b);
int64_t TYTHON_FN(bytes_islower)(TythonBytes* b);
int64_t TYTHON_FN(bytes_isspace)(TythonBytes* b);
int64_t TYTHON_FN(bytes_istitle)(TythonBytes* b);
int64_t TYTHON_FN(bytes_isupper)(TythonBytes* b);
TythonBytes* TYTHON_FN(bytes_join)(TythonBytes* sep, void* parts);
TythonBytes* TYTHON_FN(bytes_ljust)(TythonBytes* b, int64_t width, TythonBytes* fill);
TythonBytes* TYTHON_FN(bytes_lower)(TythonBytes* b);
TythonBytes* TYTHON_FN(bytes_lstrip)(TythonBytes* b, TythonBytes* chars);
TythonBytes* TYTHON_FN(bytes_maketrans)(TythonBytes* _self, TythonBytes* from, TythonBytes* to);
void* TYTHON_FN(bytes_partition)(TythonBytes* b, TythonBytes* sep);
TythonBytes* TYTHON_FN(bytes_removeprefix)(TythonBytes* b, TythonBytes* prefix);
TythonBytes* TYTHON_FN(bytes_removesuffix)(TythonBytes* b, TythonBytes* suffix);
TythonBytes* TYTHON_FN(bytes_replace)(TythonBytes* b, TythonBytes* old_sub, TythonBytes* new_sub);
int64_t TYTHON_FN(bytes_rfind)(TythonBytes* b, TythonBytes* sub);
int64_t TYTHON_FN(bytes_rindex)(TythonBytes* b, TythonBytes* sub);
TythonBytes* TYTHON_FN(bytes_rjust)(TythonBytes* b, int64_t width, TythonBytes* fill);
void* TYTHON_FN(bytes_rpartition)(TythonBytes* b, TythonBytes* sep);
void* TYTHON_FN(bytes_rsplit)(TythonBytes* b, TythonBytes* sep);
TythonBytes* TYTHON_FN(bytes_rstrip)(TythonBytes* b, TythonBytes* chars);
void* TYTHON_FN(bytes_split)(TythonBytes* b, TythonBytes* sep);
void* TYTHON_FN(bytes_splitlines)(TythonBytes* b);
int64_t TYTHON_FN(bytes_startswith)(TythonBytes* b, TythonBytes* prefix);
TythonBytes* TYTHON_FN(bytes_strip)(TythonBytes* b, TythonBytes* chars);
TythonBytes* TYTHON_FN(bytes_swapcase)(TythonBytes* b);
TythonBytes* TYTHON_FN(bytes_title)(TythonBytes* b);
TythonBytes* TYTHON_FN(bytes_translate)(TythonBytes* b, TythonBytes* table);
TythonBytes* TYTHON_FN(bytes_upper)(TythonBytes* b);
TythonBytes* TYTHON_FN(bytes_zfill)(TythonBytes* b, int64_t width);
void print_bytes_repr(const uint8_t* data, int64_t len);
int64_t bytes_repr_body_len(const uint8_t* data, int64_t len);
char* bytes_repr_body_write(char* out, const uint8_t* data, int64_t len);

#ifdef __cplusplus
}
#endif

#endif /* TYTHON_DATASTRUCTURES_BYTES_H */
