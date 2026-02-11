#include "tython.h"

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
