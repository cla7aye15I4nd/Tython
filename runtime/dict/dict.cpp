#include "tython.h"

#include <cstdio>
#include <cstdlib>
#include <cstring>

static int64_t find_key(TythonDict* d, int64_t key) {
    for (int64_t i = 0; i < d->len; i++) {
        if (d->keys[i] == key) {
            return i;
        }
    }
    return -1;
}

static void ensure_capacity(TythonDict* d, int64_t needed) {
    if (d->capacity >= needed) return;
    int64_t next = d->capacity == 0 ? 4 : d->capacity * 2;
    while (next < needed) next *= 2;

    auto* next_keys = static_cast<int64_t*>(std::malloc(sizeof(int64_t) * next));
    auto* next_values = static_cast<int64_t*>(std::malloc(sizeof(int64_t) * next));
    if (!next_keys || !next_values) {
        std::fprintf(stderr, "MemoryError: allocation failed\n");
        std::exit(1);
    }

    if (d->len > 0) {
        std::memcpy(next_keys, d->keys, sizeof(int64_t) * d->len);
        std::memcpy(next_values, d->values, sizeof(int64_t) * d->len);
    }
    std::free(d->keys);
    std::free(d->values);
    d->keys = next_keys;
    d->values = next_values;
    d->capacity = next;
}

TythonDict* TYTHON_FN(dict_empty)(void) {
    auto* d = static_cast<TythonDict*>(std::malloc(sizeof(TythonDict)));
    if (!d) {
        std::fprintf(stderr, "MemoryError: allocation failed\n");
        std::exit(1);
    }
    d->len = 0;
    d->capacity = 0;
    d->keys = nullptr;
    d->values = nullptr;
    return d;
}

int64_t TYTHON_FN(dict_len)(TythonDict* d) { return d->len; }

int64_t TYTHON_FN(dict_contains)(TythonDict* d, int64_t key) { return find_key(d, key) >= 0; }

int64_t TYTHON_FN(dict_get)(TythonDict* d, int64_t key) {
    int64_t idx = find_key(d, key);
    if (idx < 0) {
        TYTHON_FN(raise)(TYTHON_EXC_KEY_ERROR, TYTHON_FN(str_new)("key not found", 13));
        __builtin_unreachable();
    }
    return d->values[idx];
}

void TYTHON_FN(dict_set)(TythonDict* d, int64_t key, int64_t value) {
    int64_t idx = find_key(d, key);
    if (idx >= 0) {
        d->values[idx] = value;
        return;
    }
    ensure_capacity(d, d->len + 1);
    d->keys[d->len] = key;
    d->values[d->len] = value;
    d->len += 1;
}

void TYTHON_FN(dict_clear)(TythonDict* d) { d->len = 0; }

int64_t TYTHON_FN(dict_pop)(TythonDict* d, int64_t key) {
    int64_t idx = find_key(d, key);
    if (idx < 0) {
        TYTHON_FN(raise)(TYTHON_EXC_KEY_ERROR, TYTHON_FN(str_new)("key not found", 13));
        __builtin_unreachable();
    }
    int64_t out = d->values[idx];
    for (int64_t i = idx + 1; i < d->len; i++) {
        d->keys[i - 1] = d->keys[i];
        d->values[i - 1] = d->values[i];
    }
    d->len -= 1;
    return out;
}

int64_t TYTHON_FN(dict_eq)(TythonDict* a, TythonDict* b) {
    if (a == b) return 1;
    if (a->len != b->len) return 0;
    for (int64_t i = 0; i < a->len; i++) {
        int64_t key = a->keys[i];
        int64_t bi = find_key(b, key);
        if (bi < 0) return 0;
        if (a->values[i] != b->values[bi]) return 0;
    }
    return 1;
}

TythonDict* TYTHON_FN(dict_copy)(TythonDict* d) {
    auto* out = TYTHON_FN(dict_empty)();
    ensure_capacity(out, d->len);
    out->len = d->len;
    if (d->len > 0) {
        std::memcpy(out->keys, d->keys, sizeof(int64_t) * d->len);
        std::memcpy(out->values, d->values, sizeof(int64_t) * d->len);
    }
    return out;
}

