#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>
#include <math.h>

void __tython_print_int(int64_t value) {
    printf("%lld", value);
}

void __tython_print_float(double value) {
    /* Match Python's float repr: whole floats print with ".0" */
    char buf[64];
    snprintf(buf, sizeof(buf), "%.12g", value);
    int has_dot = 0;
    for (int i = 0; buf[i]; i++) {
        if (buf[i] == '.' || buf[i] == 'e' || buf[i] == 'E'
            || buf[i] == 'n' || buf[i] == 'i') {
            has_dot = 1;
            break;
        }
    }
    printf("%s", buf);
    if (!has_dot) {
        printf(".0");
    }
}

void __tython_print_bool(int64_t value) {
    if (value) {
        printf("True");
    } else {
        printf("False");
    }
}

void __tython_print_space() {
    putchar(' ');
}

void __tython_print_newline() {
    putchar('\n');
}

void __tython_assert(int64_t condition) {
    if (!condition) {
        fprintf(stderr, "AssertionError\n");
        exit(1);
    }
}

int64_t __tython_pow_int(int64_t base, int64_t exp) {
    if (exp < 0) {
        return 0;
    }
    int64_t result = 1;
    while (exp > 0) {
        if (exp & 1) {
            result *= base;
        }
        base *= base;
        exp >>= 1;
    }
    return result;
}

int64_t __tython_abs_int(int64_t x) {
    return x < 0 ? -x : x;
}

double __tython_abs_float(double x) {
    return fabs(x);
}

#define DEFINE_MINMAX(name, type, op) \
    type __tython_##name(type a, type b) { return (a op b) ? a : b; }

DEFINE_MINMAX(min_int,   int64_t, <)
DEFINE_MINMAX(min_float, double,  <)
DEFINE_MINMAX(max_int,   int64_t, >)
DEFINE_MINMAX(max_float, double,  >)

#undef DEFINE_MINMAX

int64_t __tython_round_float(double x) {
    return (int64_t)round(x);
}

void* __tython_malloc(int64_t size) {
    void* ptr = malloc((size_t)size);
    if (!ptr) {
        fprintf(stderr, "MemoryError: allocation failed\n");
        exit(1);
    }
    return ptr;
}

/* ── str type ──────────────────────────────────────────────────────── */

typedef struct {
    int64_t len;
    char* data;
} TythonStr;

TythonStr* __tython_str_new(const char* data, int64_t len) {
    TythonStr* s = (TythonStr*)__tython_malloc(sizeof(TythonStr));
    s->len = len;
    s->data = (char*)__tython_malloc(len);
    memcpy(s->data, data, (size_t)len);
    return s;
}

TythonStr* __tython_str_concat(TythonStr* a, TythonStr* b) {
    int64_t new_len = a->len + b->len;
    TythonStr* s = (TythonStr*)__tython_malloc(sizeof(TythonStr));
    s->len = new_len;
    s->data = (char*)__tython_malloc(new_len);
    memcpy(s->data, a->data, (size_t)a->len);
    memcpy(s->data + a->len, b->data, (size_t)b->len);
    return s;
}

TythonStr* __tython_str_repeat(TythonStr* s, int64_t n) {
    if (n <= 0) {
        return __tython_str_new("", 0);
    }
    int64_t new_len = s->len * n;
    TythonStr* r = (TythonStr*)__tython_malloc(sizeof(TythonStr));
    r->len = new_len;
    r->data = (char*)__tython_malloc(new_len);
    for (int64_t i = 0; i < n; i++) {
        memcpy(r->data + i * s->len, s->data, (size_t)s->len);
    }
    return r;
}

int64_t __tython_str_len(TythonStr* s) {
    return s->len;
}

int64_t __tython_str_cmp(TythonStr* a, TythonStr* b) {
    int64_t min_len = a->len < b->len ? a->len : b->len;
    int c = memcmp(a->data, b->data, (size_t)min_len);
    if (c != 0) return c < 0 ? -1 : 1;
    if (a->len < b->len) return -1;
    if (a->len > b->len) return 1;
    return 0;
}

int64_t __tython_str_eq(TythonStr* a, TythonStr* b) {
    if (a->len != b->len) return 0;
    return memcmp(a->data, b->data, (size_t)a->len) == 0 ? 1 : 0;
}

void __tython_print_str(TythonStr* s) {
    fwrite(s->data, 1, (size_t)s->len, stdout);
}

TythonStr* __tython_str_from_int(int64_t v) {
    char buf[32];
    int n = snprintf(buf, sizeof(buf), "%lld", v);
    return __tython_str_new(buf, n);
}

TythonStr* __tython_str_from_float(double v) {
    char buf[64];
    snprintf(buf, sizeof(buf), "%.12g", v);
    /* Match Python: ensure ".0" for whole floats */
    int has_dot = 0;
    for (int i = 0; buf[i]; i++) {
        if (buf[i] == '.' || buf[i] == 'e' || buf[i] == 'E'
            || buf[i] == 'n' || buf[i] == 'i') {
            has_dot = 1;
            break;
        }
    }
    if (!has_dot) {
        size_t len = strlen(buf);
        buf[len] = '.';
        buf[len + 1] = '0';
        buf[len + 2] = '\0';
    }
    return __tython_str_new(buf, (int64_t)strlen(buf));
}

TythonStr* __tython_str_from_bool(int64_t v) {
    if (v) {
        return __tython_str_new("True", 4);
    } else {
        return __tython_str_new("False", 5);
    }
}

/* ── bytes type ────────────────────────────────────────────────────── */

typedef struct {
    int64_t len;
    uint8_t* data;
} TythonBytes;

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

static void print_bytes_repr(const uint8_t* data, int64_t len) {
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

/* ── bytearray type ────────────────────────────────────────────────── */

typedef struct {
    int64_t len;
    int64_t capacity;
    uint8_t* data;
} TythonByteArray;

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
