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
void print_bytes_repr(const uint8_t* data, int64_t len);
int64_t bytes_repr_body_len(const uint8_t* data, int64_t len);
char* bytes_repr_body_write(char* out, const uint8_t* data, int64_t len);

#ifdef __cplusplus
}
#endif

#endif /* TYTHON_DATASTRUCTURES_BYTES_H */
