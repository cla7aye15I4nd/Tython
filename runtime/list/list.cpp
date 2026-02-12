#include "tython.h"
#include "internal/vec.h"

#include <cstdio>
#include <cstring>

using ListVec = tython::Vec<int64_t>;

static_assert(sizeof(ListVec) == sizeof(TythonList),
              "Vec<int64_t> must be layout-compatible with TythonList");

static auto* v(TythonList* p)  { return reinterpret_cast<ListVec*>(p); }
static auto* L(ListVec* p)     { return reinterpret_cast<TythonList*>(p); }

static int64_t resolve_index(int64_t len, int64_t index) {
    int64_t r = index;
    if (r < 0) r += len;
    if (r < 0 || r >= len) {
        std::fprintf(stderr, "IndexError: list index out of range\n");
        std::exit(1);
    }
    return r;
}

/* ── core operations (delegated to Vec<int64_t>) ─────────────────── */

TythonList* TYTHON_FN(list_new)(const int64_t* data, int64_t len) {
    return L(ListVec::create(data, len));
}

TythonList* TYTHON_FN(list_empty)(void) {
    return L(ListVec::empty());
}

int64_t TYTHON_FN(list_len)(TythonList* lst) { return v(lst)->len; }

int64_t TYTHON_FN(list_get)(TythonList* lst, int64_t index) {
    return v(lst)->data[resolve_index(v(lst)->len, index)];
}

void TYTHON_FN(list_set)(TythonList* lst, int64_t index, int64_t value) {
    v(lst)->data[resolve_index(v(lst)->len, index)] = value;
}

void    TYTHON_FN(list_append)(TythonList* lst, int64_t value) { v(lst)->push(value); }
int64_t TYTHON_FN(list_pop)(TythonList* lst) {
    if (v(lst)->len == 0) {
        std::fprintf(stderr, "IndexError: pop from empty list\n");
        std::exit(1);
    }
    return v(lst)->pop_back();
}
void TYTHON_FN(list_clear)(TythonList* lst) { v(lst)->clear(); }

/* ── queries ─────────────────────────────────────────────────────── */

int64_t TYTHON_FN(list_contains)(TythonList* lst, int64_t value) {
    return v(lst)->contains(value);
}

int64_t TYTHON_FN(list_index)(TythonList* lst, int64_t value) {
    int64_t idx = v(lst)->index_of(value);
    if (idx < 0) {
        TYTHON_FN(raise)(TYTHON_EXC_VALUE_ERROR,
                         TYTHON_FN(str_new)("x not in list", 13));
        __builtin_unreachable();
    }
    return idx;
}

int64_t TYTHON_FN(list_count)(TythonList* lst, int64_t value) {
    return v(lst)->count_of(value);
}

/* ── mutation ────────────────────────────────────────────────────── */

void TYTHON_FN(list_insert)(TythonList* lst, int64_t index, int64_t value) {
    v(lst)->insert_at(index, value);
}

void TYTHON_FN(list_remove)(TythonList* lst, int64_t value) {
    if (!v(lst)->remove_first(value)) {
        TYTHON_FN(raise)(TYTHON_EXC_VALUE_ERROR,
                         TYTHON_FN(str_new)("list.remove(x): x not in list", 30));
        __builtin_unreachable();
    }
}

void TYTHON_FN(list_reverse)(TythonList* lst) { v(lst)->reverse(); }

/* ── sorting (std::sort with typed comparators) ──────────────────── */

void TYTHON_FN(list_sort_int)(TythonList* lst) { v(lst)->sort(); }

void TYTHON_FN(list_sort_float)(TythonList* lst) {
    v(lst)->sort([](int64_t a, int64_t b) {
        double va, vb;
        std::memcpy(&va, &a, sizeof(double));
        std::memcpy(&vb, &b, sizeof(double));
        return va < vb;
    });
}

void TYTHON_FN(list_sort_str)(TythonList* lst) {
    v(lst)->sort([](int64_t a, int64_t b) {
        auto* sa = reinterpret_cast<TythonStr*>(static_cast<uintptr_t>(a));
        auto* sb = reinterpret_cast<TythonStr*>(static_cast<uintptr_t>(b));
        return TYTHON_FN(str_cmp)(sa, sb) < 0;
    });
}

void TYTHON_FN(list_sort_bytes)(TythonList* lst) {
    v(lst)->sort([](int64_t a, int64_t b) {
        auto* ba = reinterpret_cast<TythonBytes*>(static_cast<uintptr_t>(a));
        auto* bb = reinterpret_cast<TythonBytes*>(static_cast<uintptr_t>(b));
        return TYTHON_FN(bytes_cmp)(ba, bb) < 0;
    });
}

void TYTHON_FN(list_sort_bytearray)(TythonList* lst) {
    v(lst)->sort([](int64_t a, int64_t b) {
        auto* ba = reinterpret_cast<TythonByteArray*>(static_cast<uintptr_t>(a));
        auto* bb = reinterpret_cast<TythonByteArray*>(static_cast<uintptr_t>(b));
        return TYTHON_FN(bytearray_cmp)(ba, bb) < 0;
    });
}

/* ── sorted (copy + sort) ────────────────────────────────────────── */

TythonList* TYTHON_FN(sorted_int)(TythonList* lst) {
    auto* out = L(v(lst)->copy());
    TYTHON_FN(list_sort_int)(out);
    return out;
}

TythonList* TYTHON_FN(sorted_float)(TythonList* lst) {
    auto* out = L(v(lst)->copy());
    TYTHON_FN(list_sort_float)(out);
    return out;
}

TythonList* TYTHON_FN(sorted_str)(TythonList* lst) {
    auto* out = L(v(lst)->copy());
    TYTHON_FN(list_sort_str)(out);
    return out;
}

TythonList* TYTHON_FN(sorted_bytes)(TythonList* lst) {
    auto* out = L(v(lst)->copy());
    TYTHON_FN(list_sort_bytes)(out);
    return out;
}

TythonList* TYTHON_FN(sorted_bytearray)(TythonList* lst) {
    auto* out = L(v(lst)->copy());
    TYTHON_FN(list_sort_bytearray)(out);
    return out;
}

/* ── bulk operations ─────────────────────────────────────────────── */

void TYTHON_FN(list_extend)(TythonList* lst, TythonList* other) {
    v(lst)->extend_from(v(other)->data, v(other)->len);
}

TythonList* TYTHON_FN(list_copy)(TythonList* lst) {
    return L(v(lst)->copy());
}

/* ── aggregate builtins ──────────────────────────────────────────── */

int64_t TYTHON_FN(sum_int)(TythonList* lst) {
    int64_t sum = 0;
    auto* p = v(lst);
    for (int64_t i = 0; i < p->len; i++) sum += p->data[i];
    return sum;
}

double TYTHON_FN(sum_float)(TythonList* lst) {
    double sum = 0.0;
    auto* p = v(lst);
    for (int64_t i = 0; i < p->len; i++) {
        double val;
        std::memcpy(&val, &p->data[i], sizeof(double));
        sum += val;
    }
    return sum;
}

int64_t TYTHON_FN(sum_int_start)(TythonList* lst, int64_t start) {
    return start + TYTHON_FN(sum_int)(lst);
}

double TYTHON_FN(sum_float_start)(TythonList* lst, double start) {
    return start + TYTHON_FN(sum_float)(lst);
}

int64_t TYTHON_FN(all_list)(TythonList* lst) {
    auto* p = v(lst);
    for (int64_t i = 0; i < p->len; i++)
        if (p->data[i] == 0) return 0;
    return 1;
}

int64_t TYTHON_FN(any_list)(TythonList* lst) {
    auto* p = v(lst);
    for (int64_t i = 0; i < p->len; i++)
        if (p->data[i] != 0) return 1;
    return 0;
}

/* ── equality ────────────────────────────────────────────────────── */

int64_t TYTHON_FN(list_eq_shallow)(TythonList* a, TythonList* b) {
    return v(a)->eq(v(b));
}

int64_t TYTHON_FN(list_eq_deep)(TythonList* a, TythonList* b, int64_t depth) {
    if (a == b) return 1;
    if (v(a)->len != v(b)->len) return 0;
    if (depth <= 0) return v(a)->eq(v(b));
    for (int64_t i = 0; i < v(a)->len; i++) {
        auto* ai = reinterpret_cast<TythonList*>(static_cast<uintptr_t>(v(a)->data[i]));
        auto* bi = reinterpret_cast<TythonList*>(static_cast<uintptr_t>(v(b)->data[i]));
        if (!TYTHON_FN(list_eq_deep)(ai, bi, depth - 1)) return 0;
    }
    return 1;
}

