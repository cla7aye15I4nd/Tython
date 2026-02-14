#include "tython.h"
#include "gc/gc.h"

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

static int64_t find_value_by_tag(TythonSet* s, int64_t value, int64_t eq_tag) {
    for (int64_t i = 0; i < s->len; i++) {
        if (TYTHON_FN(intrinsic_eq)(eq_tag, s->data[i], value) != 0) {
            return i;
        }
    }
    return -1;
}

static void ensure_capacity(TythonSet* s, int64_t needed) {
    if (s->capacity >= needed) return;
    int64_t next = s->capacity == 0 ? 4 : s->capacity * 2;
    while (next < needed) next *= 2;
    auto* next_data = static_cast<int64_t*>(__tython_gc_malloc(sizeof(int64_t) * next));
    if (s->len > 0) {
        std::memcpy(next_data, s->data, sizeof(int64_t) * s->len);
    }
    __tython_gc_free(s->data);
    s->data = next_data;
    s->capacity = next;
}

TythonSet* TYTHON_FN(set_empty)(void) {
    auto* s = static_cast<TythonSet*>(__tython_gc_malloc(sizeof(TythonSet)));
    s->len = 0;
    s->capacity = 0;
    s->data = nullptr;
    return s;
}

int64_t TYTHON_FN(set_len)(TythonSet* s) { return s->len; }

int64_t TYTHON_FN(set_contains)(TythonSet* s, int64_t value) { return find_value(s, value) >= 0; }

int64_t TYTHON_FN(set_contains_by_tag)(TythonSet* s, int64_t value, int64_t eq_tag) {
    return find_value_by_tag(s, value, eq_tag) >= 0;
}

void TYTHON_FN(set_add)(TythonSet* s, int64_t value) {
    if (find_value(s, value) >= 0) return;
    ensure_capacity(s, s->len + 1);
    s->data[s->len] = value;
    s->len += 1;
}

void TYTHON_FN(set_add_by_tag)(TythonSet* s, int64_t value, int64_t eq_tag) {
    if (find_value_by_tag(s, value, eq_tag) >= 0) return;
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

void TYTHON_FN(set_remove_by_tag)(TythonSet* s, int64_t value, int64_t eq_tag) {
    int64_t idx = find_value_by_tag(s, value, eq_tag);
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

void TYTHON_FN(set_discard_by_tag)(TythonSet* s, int64_t value, int64_t eq_tag) {
    int64_t idx = find_value_by_tag(s, value, eq_tag);
    if (idx < 0) return;
    for (int64_t i = idx + 1; i < s->len; i++) {
        s->data[i - 1] = s->data[i];
    }
    s->len -= 1;
}

TythonSet* TYTHON_FN(set_union_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_tag) {
    auto* out = TYTHON_FN(set_copy)(a);
    for (int64_t i = 0; i < b->len; i++) {
        TYTHON_FN(set_add_by_tag)(out, b->data[i], eq_tag);
    }
    return out;
}

void TYTHON_FN(set_update_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_tag) {
    for (int64_t i = 0; i < b->len; i++) {
        TYTHON_FN(set_add_by_tag)(a, b->data[i], eq_tag);
    }
}

TythonSet* TYTHON_FN(set_intersection_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_tag) {
    auto* out = TYTHON_FN(set_empty)();
    for (int64_t i = 0; i < a->len; i++) {
        if (find_value_by_tag(b, a->data[i], eq_tag) >= 0) {
            TYTHON_FN(set_add_by_tag)(out, a->data[i], eq_tag);
        }
    }
    return out;
}

void TYTHON_FN(set_intersection_update_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_tag) {
    int64_t write = 0;
    for (int64_t i = 0; i < a->len; i++) {
        if (find_value_by_tag(b, a->data[i], eq_tag) >= 0) {
            a->data[write++] = a->data[i];
        }
    }
    a->len = write;
}

TythonSet* TYTHON_FN(set_difference_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_tag) {
    auto* out = TYTHON_FN(set_empty)();
    for (int64_t i = 0; i < a->len; i++) {
        if (find_value_by_tag(b, a->data[i], eq_tag) < 0) {
            TYTHON_FN(set_add_by_tag)(out, a->data[i], eq_tag);
        }
    }
    return out;
}

void TYTHON_FN(set_difference_update_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_tag) {
    int64_t write = 0;
    for (int64_t i = 0; i < a->len; i++) {
        if (find_value_by_tag(b, a->data[i], eq_tag) < 0) {
            a->data[write++] = a->data[i];
        }
    }
    a->len = write;
}

TythonSet* TYTHON_FN(set_symmetric_difference_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_tag) {
    auto* out = TYTHON_FN(set_empty)();
    for (int64_t i = 0; i < a->len; i++) {
        if (find_value_by_tag(b, a->data[i], eq_tag) < 0) {
            TYTHON_FN(set_add_by_tag)(out, a->data[i], eq_tag);
        }
    }
    for (int64_t i = 0; i < b->len; i++) {
        if (find_value_by_tag(a, b->data[i], eq_tag) < 0) {
            TYTHON_FN(set_add_by_tag)(out, b->data[i], eq_tag);
        }
    }
    return out;
}

void TYTHON_FN(set_symmetric_difference_update_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_tag) {
    auto* out = TYTHON_FN(set_symmetric_difference_by_tag)(a, b, eq_tag);
    ensure_capacity(a, out->len);
    a->len = out->len;
    if (out->len > 0) {
        std::memcpy(a->data, out->data, static_cast<size_t>(out->len) * sizeof(int64_t));
    }
}

int64_t TYTHON_FN(set_isdisjoint_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_tag) {
    for (int64_t i = 0; i < a->len; i++) {
        if (find_value_by_tag(b, a->data[i], eq_tag) >= 0) {
            return 0;
        }
    }
    return 1;
}

int64_t TYTHON_FN(set_issubset_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_tag) {
    for (int64_t i = 0; i < a->len; i++) {
        if (find_value_by_tag(b, a->data[i], eq_tag) < 0) {
            return 0;
        }
    }
    return 1;
}

int64_t TYTHON_FN(set_issuperset_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_tag) {
    return TYTHON_FN(set_issubset_by_tag)(b, a, eq_tag);
}

int64_t TYTHON_FN(set_lt_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_tag) {
    return a->len < b->len && TYTHON_FN(set_issubset_by_tag)(a, b, eq_tag);
}

int64_t TYTHON_FN(set_le_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_tag) {
    return TYTHON_FN(set_issubset_by_tag)(a, b, eq_tag);
}

int64_t TYTHON_FN(set_gt_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_tag) {
    return TYTHON_FN(set_lt_by_tag)(b, a, eq_tag);
}

int64_t TYTHON_FN(set_ge_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_tag) {
    return TYTHON_FN(set_le_by_tag)(b, a, eq_tag);
}

TythonSet* TYTHON_FN(set_iand_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_tag) {
    TYTHON_FN(set_intersection_update_by_tag)(a, b, eq_tag);
    return a;
}

TythonSet* TYTHON_FN(set_ior_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_tag) {
    TYTHON_FN(set_update_by_tag)(a, b, eq_tag);
    return a;
}

TythonSet* TYTHON_FN(set_isub_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_tag) {
    TYTHON_FN(set_difference_update_by_tag)(a, b, eq_tag);
    return a;
}

TythonSet* TYTHON_FN(set_ixor_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_tag) {
    TYTHON_FN(set_symmetric_difference_update_by_tag)(a, b, eq_tag);
    return a;
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

int64_t TYTHON_FN(set_eq_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_tag) {
    if (a == b) return 1;
    if (a->len != b->len) return 0;
    for (int64_t i = 0; i < a->len; i++) {
        if (find_value_by_tag(b, a->data[i], eq_tag) < 0) return 0;
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
