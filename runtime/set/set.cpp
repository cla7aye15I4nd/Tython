#include "tython.h"

#include <cstdio>
#include <cstdlib>
#include <cstring>

static int64_t find_value(TythonSet* s, int64_t value) {
    for (int64_t i = 0; i < s->len; i++) {
        if (s->data[i] == value) {
            return i;
        }
    }
    return -1;
}

static void ensure_capacity(TythonSet* s, int64_t needed) {
    if (s->capacity >= needed) return;
    int64_t next = s->capacity == 0 ? 4 : s->capacity * 2;
    while (next < needed) next *= 2;
    auto* next_data = static_cast<int64_t*>(std::malloc(sizeof(int64_t) * next));
    if (!next_data) {
        std::fprintf(stderr, "MemoryError: allocation failed\n");
        std::exit(1);
    }
    if (s->len > 0) {
        std::memcpy(next_data, s->data, sizeof(int64_t) * s->len);
    }
    std::free(s->data);
    s->data = next_data;
    s->capacity = next;
}

TythonSet* TYTHON_FN(set_empty)(void) {
    auto* s = static_cast<TythonSet*>(std::malloc(sizeof(TythonSet)));
    if (!s) {
        std::fprintf(stderr, "MemoryError: allocation failed\n");
        std::exit(1);
    }
    s->len = 0;
    s->capacity = 0;
    s->data = nullptr;
    return s;
}

int64_t TYTHON_FN(set_len)(TythonSet* s) { return s->len; }

int64_t TYTHON_FN(set_contains)(TythonSet* s, int64_t value) { return find_value(s, value) >= 0; }

void TYTHON_FN(set_add)(TythonSet* s, int64_t value) {
    if (find_value(s, value) >= 0) return;
    ensure_capacity(s, s->len + 1);
    s->data[s->len] = value;
    s->len += 1;
}

void TYTHON_FN(set_remove)(TythonSet* s, int64_t value) {
    int64_t idx = find_value(s, value);
    if (idx < 0) {
        TYTHON_FN(raise)(TYTHON_EXC_KEY_ERROR, TYTHON_FN(str_new)("value not found", 15));
        __builtin_unreachable();
    }
    for (int64_t i = idx + 1; i < s->len; i++) {
        s->data[i - 1] = s->data[i];
    }
    s->len -= 1;
}

void TYTHON_FN(set_discard)(TythonSet* s, int64_t value) {
    int64_t idx = find_value(s, value);
    if (idx < 0) return;
    for (int64_t i = idx + 1; i < s->len; i++) {
        s->data[i - 1] = s->data[i];
    }
    s->len -= 1;
}

int64_t TYTHON_FN(set_pop)(TythonSet* s) {
    if (s->len == 0) {
        TYTHON_FN(raise)(TYTHON_EXC_KEY_ERROR, TYTHON_FN(str_new)("pop from empty set", 18));
        __builtin_unreachable();
    }
    int64_t out = s->data[s->len - 1];
    s->len -= 1;
    return out;
}

void TYTHON_FN(set_clear)(TythonSet* s) { s->len = 0; }

int64_t TYTHON_FN(set_eq)(TythonSet* a, TythonSet* b) {
    if (a == b) return 1;
    if (a->len != b->len) return 0;
    for (int64_t i = 0; i < a->len; i++) {
        if (find_value(b, a->data[i]) < 0) return 0;
    }
    return 1;
}

TythonSet* TYTHON_FN(set_copy)(TythonSet* s) {
    auto* out = TYTHON_FN(set_empty)();
    ensure_capacity(out, s->len);
    out->len = s->len;
    if (s->len > 0) {
        std::memcpy(out->data, s->data, sizeof(int64_t) * s->len);
    }
    return out;
}

