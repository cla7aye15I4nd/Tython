#include "tython.h"
#include "gc/gc.h"

#include <cstdio>
#include <cstdlib>
#include <cstring>

struct TythonDictItemPair {
    int64_t key;
    int64_t value;
};

static int64_t make_item_pair_slot(int64_t key, int64_t value) {
    auto* pair = static_cast<TythonDictItemPair*>(__tython_gc_malloc(sizeof(TythonDictItemPair)));
    pair->key = key;
    pair->value = value;
    return reinterpret_cast<int64_t>(pair);
}

static int64_t find_key(TythonDict* d, int64_t key) {
    for (int64_t i = 0; i < d->len; i++) {
        if (d->keys[i] == key) {
            return i;
        }
    }
    return -1;
}

static int64_t find_key_by_tag(TythonDict* d, int64_t key, int64_t key_eq_tag) {
    for (int64_t i = 0; i < d->len; i++) {
        if (TYTHON_FN(intrinsic_eq)(key_eq_tag, d->keys[i], key) != 0) {
            return i;
        }
    }
    return -1;
}

static void ensure_capacity(TythonDict* d, int64_t needed) {
    if (d->capacity >= needed) return;
    int64_t next = d->capacity == 0 ? 4 : d->capacity * 2;
    while (next < needed) next *= 2;

    auto* next_keys = static_cast<int64_t*>(__tython_gc_malloc(sizeof(int64_t) * next));
    auto* next_values = static_cast<int64_t*>(__tython_gc_malloc(sizeof(int64_t) * next));

    if (d->len > 0) {
        std::memcpy(next_keys, d->keys, sizeof(int64_t) * d->len);
        std::memcpy(next_values, d->values, sizeof(int64_t) * d->len);
    }
    __tython_gc_free(d->keys);
    __tython_gc_free(d->values);
    d->keys = next_keys;
    d->values = next_values;
    d->capacity = next;
}

TythonDict* TYTHON_FN(dict_empty)(void) {
    auto* d = static_cast<TythonDict*>(__tython_gc_malloc(sizeof(TythonDict)));
    d->len = 0;
    d->capacity = 0;
    d->keys = nullptr;
    d->values = nullptr;
    return d;
}

int64_t TYTHON_FN(dict_len)(TythonDict* d) { return d->len; }

int64_t TYTHON_FN(dict_contains)(TythonDict* d, int64_t key) { return find_key(d, key) >= 0; }

int64_t TYTHON_FN(dict_contains_by_tag)(TythonDict* d, int64_t key, int64_t key_eq_tag) {
    return find_key_by_tag(d, key, key_eq_tag) >= 0;
}

int64_t TYTHON_FN(dict_get)(TythonDict* d, int64_t key) {
    int64_t idx = find_key(d, key);
    if (idx < 0) {
        TYTHON_FN(raise)(TYTHON_EXC_KEY_ERROR, TYTHON_FN(str_new)("key not found", 13));
        __builtin_unreachable();
    }
    return d->values[idx];
}

int64_t TYTHON_FN(dict_get_by_tag)(TythonDict* d, int64_t key, int64_t key_eq_tag) {
    int64_t idx = find_key_by_tag(d, key, key_eq_tag);
    if (idx < 0) {
        TYTHON_FN(raise)(TYTHON_EXC_KEY_ERROR, TYTHON_FN(str_new)("key not found", 13));
        __builtin_unreachable();
    }
    return d->values[idx];
}

int64_t TYTHON_FN(dict_get_default_by_tag)(
    TythonDict* d,
    int64_t key,
    int64_t default_value,
    int64_t key_eq_tag
) {
    int64_t idx = find_key_by_tag(d, key, key_eq_tag);
    if (idx < 0) {
        return default_value;
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

void TYTHON_FN(dict_set_by_tag)(TythonDict* d, int64_t key, int64_t value, int64_t key_eq_tag) {
    int64_t idx = find_key_by_tag(d, key, key_eq_tag);
    if (idx >= 0) {
        d->values[idx] = value;
        return;
    }
    ensure_capacity(d, d->len + 1);
    d->keys[d->len] = key;
    d->values[d->len] = value;
    d->len += 1;
}

int64_t TYTHON_FN(dict_setdefault_by_tag)(
    TythonDict* d,
    int64_t key,
    int64_t default_value,
    int64_t key_eq_tag
) {
    int64_t idx = find_key_by_tag(d, key, key_eq_tag);
    if (idx >= 0) {
        return d->values[idx];
    }
    ensure_capacity(d, d->len + 1);
    d->keys[d->len] = key;
    d->values[d->len] = default_value;
    d->len += 1;
    return default_value;
}

void TYTHON_FN(dict_del_by_tag)(TythonDict* d, int64_t key, int64_t key_eq_tag) {
    int64_t idx = find_key_by_tag(d, key, key_eq_tag);
    if (idx < 0) {
        TYTHON_FN(raise)(TYTHON_EXC_KEY_ERROR, TYTHON_FN(str_new)("key not found", 13));
        __builtin_unreachable();
    }
    for (int64_t i = idx + 1; i < d->len; i++) {
        d->keys[i - 1] = d->keys[i];
        d->values[i - 1] = d->values[i];
    }
    d->len -= 1;
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

int64_t TYTHON_FN(dict_pop_by_tag)(TythonDict* d, int64_t key, int64_t key_eq_tag) {
    int64_t idx = find_key_by_tag(d, key, key_eq_tag);
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

int64_t TYTHON_FN(dict_pop_default_by_tag)(
    TythonDict* d,
    int64_t key,
    int64_t default_value,
    int64_t key_eq_tag
) {
    int64_t idx = find_key_by_tag(d, key, key_eq_tag);
    if (idx < 0) {
        return default_value;
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

int64_t TYTHON_FN(dict_eq_by_tag)(
    TythonDict* a,
    TythonDict* b,
    int64_t key_eq_tag,
    int64_t value_eq_tag
) {
    if (a == b) return 1;
    if (a->len != b->len) return 0;
    for (int64_t i = 0; i < a->len; i++) {
        int64_t key = a->keys[i];
        int64_t bi = find_key_by_tag(b, key, key_eq_tag);
        if (bi < 0) return 0;
        if (TYTHON_FN(intrinsic_eq)(value_eq_tag, a->values[i], b->values[bi]) == 0) {
            return 0;
        }
    }
    return 1;
}

void TYTHON_FN(dict_update_by_tag)(TythonDict* dst, TythonDict* src, int64_t key_eq_tag) {
    for (int64_t i = 0; i < src->len; i++) {
        TYTHON_FN(dict_set_by_tag)(dst, src->keys[i], src->values[i], key_eq_tag);
    }
}

TythonDict* TYTHON_FN(dict_or_by_tag)(TythonDict* a, TythonDict* b, int64_t key_eq_tag) {
    auto* out = TYTHON_FN(dict_copy)(a);
    TYTHON_FN(dict_update_by_tag)(out, b, key_eq_tag);
    return out;
}

TythonDict* TYTHON_FN(dict_ior_by_tag)(TythonDict* a, TythonDict* b, int64_t key_eq_tag) {
    TYTHON_FN(dict_update_by_tag)(a, b, key_eq_tag);
    return a;
}

TythonDict* TYTHON_FN(dict_fromkeys_by_tag)(void* keys, int64_t value, int64_t key_eq_tag) {
    auto* out = TYTHON_FN(dict_empty)();
    auto* key_list = static_cast<TythonList*>(keys);
    int64_t n = TYTHON_FN(list_len)(key_list);
    for (int64_t i = 0; i < n; i++) {
        int64_t key = TYTHON_FN(list_get)(key_list, i);
        TYTHON_FN(dict_set_by_tag)(out, key, value, key_eq_tag);
    }
    return out;
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

void* TYTHON_FN(dict_items)(TythonDict* d) {
    auto* slots = static_cast<int64_t*>(__tython_gc_malloc(sizeof(int64_t) * d->len));
    for (int64_t i = 0; i < d->len; i++) {
        slots[i] = make_item_pair_slot(d->keys[i], d->values[i]);
    }
    return TYTHON_FN(list_new)(slots, d->len);
}

void* TYTHON_FN(dict_popitem)(TythonDict* d) {
    if (d->len == 0) {
        TYTHON_FN(raise)(TYTHON_EXC_KEY_ERROR, TYTHON_FN(str_new)("popitem(): dictionary is empty", 30));
        __builtin_unreachable();
    }
    int64_t idx = d->len - 1;
    int64_t key = d->keys[idx];
    int64_t value = d->values[idx];
    d->len -= 1;
    return reinterpret_cast<void*>(make_item_pair_slot(key, value));
}

void* TYTHON_FN(dict_keys)(TythonDict* d) {
    return TYTHON_FN(list_new)(d->keys, d->len);
}

void* TYTHON_FN(dict_values)(TythonDict* d) {
    return TYTHON_FN(list_new)(d->values, d->len);
}
