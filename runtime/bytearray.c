#include "tython.h"

TythonByteArray* __tython_bytearray_new(const uint8_t* data, int64_t len) {
    TythonByteArray* ba = (TythonByteArray*)__tython_malloc(sizeof(TythonByteArray));
    int64_t cap = len > 8 ? len : 8;
    ba->len = len;
    ba->capacity = cap;
    ba->data = (uint8_t*)__tython_malloc(cap);
    if (len > 0 && data) {
        memcpy(ba->data, data, (size_t)len);
    }
    return ba;
}

TythonByteArray* __tython_bytearray_empty(void) {
    return __tython_bytearray_new(NULL, 0);
}

TythonByteArray* __tython_bytearray_from_int(int64_t n) {
    if (n < 0) {
        fprintf(stderr, "ValueError: negative count\n");
        exit(1);
    }
    TythonByteArray* ba = (TythonByteArray*)__tython_malloc(sizeof(TythonByteArray));
    int64_t cap = n > 8 ? n : 8;
    ba->len = n;
    ba->capacity = cap;
    ba->data = (uint8_t*)__tython_malloc(cap);
    memset(ba->data, 0, (size_t)n);
    return ba;
}

TythonByteArray* __tython_bytearray_from_bytes(TythonBytes* b) {
    return __tython_bytearray_new(b->data, b->len);
}

TythonByteArray* __tython_bytearray_concat(TythonByteArray* a, TythonByteArray* b) {
    int64_t new_len = a->len + b->len;
    TythonByteArray* r = __tython_bytearray_new(NULL, 0);
    r->len = new_len;
    int64_t cap = new_len > 8 ? new_len : 8;
    r->capacity = cap;
    free(r->data);
    r->data = (uint8_t*)__tython_malloc(cap);
    memcpy(r->data, a->data, (size_t)a->len);
    memcpy(r->data + a->len, b->data, (size_t)b->len);
    return r;
}

TythonByteArray* __tython_bytearray_repeat(TythonByteArray* s, int64_t n) {
    if (n <= 0) {
        return __tython_bytearray_new(NULL, 0);
    }
    int64_t new_len = s->len * n;
    TythonByteArray* r = (TythonByteArray*)__tython_malloc(sizeof(TythonByteArray));
    r->len = new_len;
    r->capacity = new_len;
    r->data = (uint8_t*)__tython_malloc(new_len);
    for (int64_t i = 0; i < n; i++) {
        memcpy(r->data + i * s->len, s->data, (size_t)s->len);
    }
    return r;
}

int64_t __tython_bytearray_len(TythonByteArray* ba) {
    return ba->len;
}

int64_t __tython_bytearray_cmp(TythonByteArray* a, TythonByteArray* b) {
    int64_t min_len = a->len < b->len ? a->len : b->len;
    int c = memcmp(a->data, b->data, (size_t)min_len);
    if (c != 0) return c < 0 ? -1 : 1;
    if (a->len < b->len) return -1;
    if (a->len > b->len) return 1;
    return 0;
}

int64_t __tython_bytearray_eq(TythonByteArray* a, TythonByteArray* b) {
    if (a->len != b->len) return 0;
    return memcmp(a->data, b->data, (size_t)a->len) == 0 ? 1 : 0;
}

void __tython_print_bytearray(TythonByteArray* ba) {
    printf("bytearray(");
    print_bytes_repr(ba->data, ba->len);
    printf(")");
}

void __tython_bytearray_append(TythonByteArray* ba, int64_t byte_val) {
    if (ba->len >= ba->capacity) {
        int64_t new_cap = ba->capacity * 2;
        if (new_cap < 8) new_cap = 8;
        uint8_t* new_data = (uint8_t*)__tython_malloc(new_cap);
        memcpy(new_data, ba->data, (size_t)ba->len);
        free(ba->data);
        ba->data = new_data;
        ba->capacity = new_cap;
    }
    ba->data[ba->len] = (uint8_t)(byte_val & 0xFF);
    ba->len++;
}

void __tython_bytearray_extend(TythonByteArray* ba, TythonBytes* other) {
    int64_t needed = ba->len + other->len;
    if (needed > ba->capacity) {
        int64_t new_cap = ba->capacity * 2;
        if (new_cap < needed) new_cap = needed;
        uint8_t* new_data = (uint8_t*)__tython_malloc(new_cap);
        memcpy(new_data, ba->data, (size_t)ba->len);
        free(ba->data);
        ba->data = new_data;
        ba->capacity = new_cap;
    }
    memcpy(ba->data + ba->len, other->data, (size_t)other->len);
    ba->len = needed;
}

void __tython_bytearray_clear(TythonByteArray* ba) {
    ba->len = 0;
}

void __tython_bytearray_insert(TythonByteArray* ba, int64_t index, int64_t byte_val) {
    if (index < 0) index = ba->len + index;
    if (index < 0) index = 0;
    if (index > ba->len) index = ba->len;
    if (ba->len >= ba->capacity) {
        int64_t new_cap = ba->capacity * 2;
        if (new_cap < 8) new_cap = 8;
        uint8_t* new_data = (uint8_t*)__tython_malloc(new_cap);
        memcpy(new_data, ba->data, (size_t)ba->len);
        free(ba->data);
        ba->data = new_data;
        ba->capacity = new_cap;
    }
    memmove(ba->data + index + 1, ba->data + index, (size_t)(ba->len - index));
    ba->data[index] = (uint8_t)(byte_val & 0xFF);
    ba->len++;
}

void __tython_bytearray_remove(TythonByteArray* ba, int64_t byte_val) {
    uint8_t target = (uint8_t)(byte_val & 0xFF);
    for (int64_t i = 0; i < ba->len; i++) {
        if (ba->data[i] == target) {
            memmove(ba->data + i, ba->data + i + 1, (size_t)(ba->len - i - 1));
            ba->len--;
            return;
        }
    }
    fprintf(stderr, "ValueError: value not found in bytearray\n");
    exit(1);
}

void __tython_bytearray_reverse(TythonByteArray* ba) {
    for (int64_t i = 0, j = ba->len - 1; i < j; i++, j--) {
        uint8_t tmp = ba->data[i];
        ba->data[i] = ba->data[j];
        ba->data[j] = tmp;
    }
}
