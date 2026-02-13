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

TythonList* TYTHON_FN(list_concat)(TythonList* a, TythonList* b) {
    return L(v(a)->concat(v(b)));
}

int64_t TYTHON_FN(list_len)(TythonList* lst) { return v(lst)->len; }

int64_t TYTHON_FN(list_get)(TythonList* lst, int64_t index) {
    return v(lst)->data[resolve_index(v(lst)->len, index)];
}

TythonList* TYTHON_FN(list_slice)(TythonList* lst, int64_t start, int64_t stop) {
    int64_t len = v(lst)->len;
    int64_t s = start;
    int64_t e = stop;
    if (s < 0) s += len;
    if (e < 0) e += len;
    if (s < 0) s = 0;
    if (s > len) s = len;
    if (e < 0) e = 0;
    if (e > len) e = len;
    if (e < s) e = s;
    return L(ListVec::create(v(lst)->data + s, e - s));
}

TythonList* TYTHON_FN(list_repeat)(TythonList* lst, int64_t n) {
    return L(v(lst)->repeat(n));
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

TythonList* TYTHON_FN(reversed_list)(TythonList* lst) {
    auto* out = L(v(lst)->copy());
    TYTHON_FN(list_reverse)(out);
    return out;
}

/* ── bulk operations ─────────────────────────────────────────────── */

void TYTHON_FN(list_extend)(TythonList* lst, TythonList* other) {
    v(lst)->extend_from(v(other)->data, v(other)->len);
}

TythonList* TYTHON_FN(list_copy)(TythonList* lst) {
    return L(v(lst)->copy());
}

/* ── range(...) expression builtin ───────────────────────────────── */

static TythonList* range_impl(int64_t start, int64_t stop, int64_t step) {
    if (step == 0) {
        TYTHON_FN(raise)(TYTHON_EXC_VALUE_ERROR, TYTHON_FN(str_new)("range() arg 3 must not be zero", 31));
        __builtin_unreachable();
    }
    auto* out = ListVec::empty();
    if (step > 0) {
        for (int64_t i = start; i < stop; i += step) out->push(i);
    } else {
        for (int64_t i = start; i > stop; i += step) out->push(i);
    }
    return L(out);
}

TythonList* TYTHON_FN(range_1)(int64_t stop) {
    return range_impl(0, stop, 1);
}

TythonList* TYTHON_FN(range_2)(int64_t start, int64_t stop) {
    return range_impl(start, stop, 1);
}

TythonList* TYTHON_FN(range_3)(int64_t start, int64_t stop, int64_t step) {
    return range_impl(start, stop, step);
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

int64_t TYTHON_FN(max_list_int)(TythonList* lst) {
    auto* p = v(lst);
    if (p->len <= 0) {
        TYTHON_FN(raise)(TYTHON_EXC_VALUE_ERROR, TYTHON_FN(str_new)("max() arg is an empty sequence", 30));
        __builtin_unreachable();
    }
    int64_t m = p->data[0];
    for (int64_t i = 1; i < p->len; i++) if (p->data[i] > m) m = p->data[i];
    return m;
}

double TYTHON_FN(max_list_float)(TythonList* lst) {
    auto* p = v(lst);
    if (p->len <= 0) {
        TYTHON_FN(raise)(TYTHON_EXC_VALUE_ERROR, TYTHON_FN(str_new)("max() arg is an empty sequence", 30));
        __builtin_unreachable();
    }
    double m;
    std::memcpy(&m, &p->data[0], sizeof(double));
    for (int64_t i = 1; i < p->len; i++) {
        double val;
        std::memcpy(&val, &p->data[i], sizeof(double));
        if (val > m) m = val;
    }
    return m;
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
