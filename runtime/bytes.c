#include "tython.h"

TythonBytes* __tython_bytes_new(const uint8_t* data, int64_t len) {
    TythonBytes* b = (TythonBytes*)__tython_malloc(sizeof(TythonBytes));
    b->len = len;
    b->data = (uint8_t*)__tython_malloc(len > 0 ? len : 1);
    if (len > 0) {
        memcpy(b->data, data, (size_t)len);
    }
    return b;
}

TythonBytes* __tython_bytes_concat(TythonBytes* a, TythonBytes* b) {
    int64_t new_len = a->len + b->len;
    TythonBytes* r = (TythonBytes*)__tython_malloc(sizeof(TythonBytes));
    r->len = new_len;
    r->data = (uint8_t*)__tython_malloc(new_len > 0 ? new_len : 1);
    memcpy(r->data, a->data, (size_t)a->len);
    memcpy(r->data + a->len, b->data, (size_t)b->len);
    return r;
}

TythonBytes* __tython_bytes_repeat(TythonBytes* s, int64_t n) {
    if (n <= 0) {
        return __tython_bytes_new(NULL, 0);
    }
    int64_t new_len = s->len * n;
    TythonBytes* r = (TythonBytes*)__tython_malloc(sizeof(TythonBytes));
    r->len = new_len;
    r->data = (uint8_t*)__tython_malloc(new_len);
    for (int64_t i = 0; i < n; i++) {
        memcpy(r->data + i * s->len, s->data, (size_t)s->len);
    }
    return r;
}

int64_t __tython_bytes_len(TythonBytes* b) {
    return b->len;
}

int64_t __tython_bytes_cmp(TythonBytes* a, TythonBytes* b) {
    int64_t min_len = a->len < b->len ? a->len : b->len;
    int c = memcmp(a->data, b->data, (size_t)min_len);
    if (c != 0) return c < 0 ? -1 : 1;
    if (a->len < b->len) return -1;
    if (a->len > b->len) return 1;
    return 0;
}

int64_t __tython_bytes_eq(TythonBytes* a, TythonBytes* b) {
    if (a->len != b->len) return 0;
    return memcmp(a->data, b->data, (size_t)a->len) == 0 ? 1 : 0;
}

void print_bytes_repr(const uint8_t* data, int64_t len) {
    putchar('b');
    putchar('\'');
    for (int64_t i = 0; i < len; i++) {
        uint8_t c = data[i];
        if (c == '\\') {
            putchar('\\'); putchar('\\');
        } else if (c == '\'') {
            putchar('\\'); putchar('\'');
        } else if (c == '\t') {
            putchar('\\'); putchar('t');
        } else if (c == '\n') {
            putchar('\\'); putchar('n');
        } else if (c == '\r') {
            putchar('\\'); putchar('r');
        } else if (c >= 32 && c < 127) {
            putchar(c);
        } else {
            printf("\\x%02x", c);
        }
    }
    putchar('\'');
}

void __tython_print_bytes(TythonBytes* b) {
    print_bytes_repr(b->data, b->len);
}

TythonBytes* __tython_bytes_from_int(int64_t n) {
    if (n < 0) {
        fprintf(stderr, "ValueError: negative count\n");
        exit(1);
    }
    TythonBytes* b = (TythonBytes*)__tython_malloc(sizeof(TythonBytes));
    b->len = n;
    b->data = (uint8_t*)__tython_malloc(n > 0 ? n : 1);
    memset(b->data, 0, (size_t)n);
    return b;
}

TythonBytes* __tython_bytes_from_str(TythonStr* s) {
    return __tython_bytes_new((const uint8_t*)s->data, s->len);
}
