#include "tython.h"
#include "internal/vec.h"

#include <cstdio>
#include <cstring>
#include <string>

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

TythonList* TYTHON_FN(list_iadd)(TythonList* lst, TythonList* other) {
    auto* lhs = v(lst);
    auto* rhs = v(other);
    if (lhs == rhs) {
        const int64_t orig_len = lhs->len;
        lhs->grow(orig_len * 2);
        std::memcpy(lhs->data + orig_len, lhs->data,
                    static_cast<size_t>(orig_len) * sizeof(int64_t));
        lhs->len = orig_len * 2;
        return lst;
    }

    lhs->extend_from(rhs->data, rhs->len);
    return lst;
}

TythonList* TYTHON_FN(list_imul)(TythonList* lst, int64_t n) {
    auto* p = v(lst);
    if (n <= 0 || p->len == 0) {
        p->clear();
        return lst;
    }
    if (n == 1) return lst;

    const int64_t orig_len = p->len;
    const int64_t new_len = orig_len * n;
    p->grow(new_len);
    for (int64_t i = 1; i < n; i++) {
        std::memcpy(p->data + (i * orig_len), p->data,
                    static_cast<size_t>(orig_len) * sizeof(int64_t));
    }
    p->len = new_len;
    return lst;
}

void TYTHON_FN(list_del)(TythonList* lst, int64_t index) {
    auto* p = v(lst);
    int64_t idx = resolve_index(p->len, index);
    std::memmove(p->data + idx, p->data + idx + 1,
                 static_cast<size_t>(p->len - idx - 1) * sizeof(int64_t));
    p->len--;
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

/* ── generic by-tag algorithms ───────────────────────────────────── */

static inline const TythonEqOps* eq_ops_from_handle(int64_t handle) {
    return reinterpret_cast<const TythonEqOps*>(static_cast<uintptr_t>(handle));
}

static inline const TythonLtOps* lt_ops_from_handle(int64_t handle) {
    return reinterpret_cast<const TythonLtOps*>(static_cast<uintptr_t>(handle));
}

static inline const TythonStrOps* str_ops_from_handle(int64_t handle) {
    return reinterpret_cast<const TythonStrOps*>(static_cast<uintptr_t>(handle));
}

int64_t TYTHON_FN(list_eq_by_tag)(TythonList* a, TythonList* b, int64_t eq_ops_handle) {
    if (a == b) return 1;
    auto* av = v(a);
    auto* bv = v(b);
    if (av->len != bv->len) return 0;
    const TythonEqOps* ops = eq_ops_from_handle(eq_ops_handle);
    for (int64_t i = 0; i < av->len; i++) {
        if (!ops->eq(av->data[i], bv->data[i])) return 0;
    }
    return 1;
}

int64_t TYTHON_FN(list_lt_by_tag)(TythonList* a, TythonList* b, int64_t lt_ops_handle) {
    auto* av = v(a);
    auto* bv = v(b);
    const TythonLtOps* ops = lt_ops_from_handle(lt_ops_handle);
    int64_t min_len = av->len < bv->len ? av->len : bv->len;
    for (int64_t i = 0; i < min_len; i++) {
        const int64_t lhs = av->data[i];
        const int64_t rhs = bv->data[i];
        if (ops->lt(lhs, rhs)) return 1;
        if (ops->lt(rhs, lhs)) return 0;
    }
    return av->len < bv->len ? 1 : 0;
}

int64_t TYTHON_FN(list_contains_by_tag)(TythonList* lst, int64_t value, int64_t eq_ops_handle) {
    auto* p = v(lst);
    const TythonEqOps* ops = eq_ops_from_handle(eq_ops_handle);
    for (int64_t i = 0; i < p->len; i++) {
        if (ops->eq(p->data[i], value)) return 1;
    }
    return 0;
}

int64_t TYTHON_FN(list_index_by_tag)(TythonList* lst, int64_t value, int64_t eq_ops_handle) {
    auto* p = v(lst);
    const TythonEqOps* ops = eq_ops_from_handle(eq_ops_handle);
    for (int64_t i = 0; i < p->len; i++) {
        if (ops->eq(p->data[i], value)) return i;
    }
    TYTHON_FN(raise)(TYTHON_EXC_VALUE_ERROR, TYTHON_FN(str_new)("x not in list", 13));
    __builtin_unreachable();
}

int64_t TYTHON_FN(list_count_by_tag)(TythonList* lst, int64_t value, int64_t eq_ops_handle) {
    auto* p = v(lst);
    const TythonEqOps* ops = eq_ops_from_handle(eq_ops_handle);
    int64_t out = 0;
    for (int64_t i = 0; i < p->len; i++) {
        if (ops->eq(p->data[i], value)) out++;
    }
    return out;
}

void TYTHON_FN(list_remove_by_tag)(TythonList* lst, int64_t value, int64_t eq_ops_handle) {
    int64_t idx = TYTHON_FN(list_index_by_tag)(lst, value, eq_ops_handle);
    auto* p = v(lst);
    for (int64_t i = idx + 1; i < p->len; i++) {
        p->data[i - 1] = p->data[i];
    }
    p->len -= 1;
}

void TYTHON_FN(list_sort_by_tag)(TythonList* lst, int64_t lt_ops_handle) {
    auto* p = v(lst);
    const TythonLtOps* ops = lt_ops_from_handle(lt_ops_handle);
    for (int64_t i = 1; i < p->len; i++) {
        int64_t key = p->data[i];
        int64_t j = i - 1;
        while (j >= 0 && ops->lt(key, p->data[j])) {
            p->data[j + 1] = p->data[j];
            j -= 1;
        }
        p->data[j + 1] = key;
    }
}

TythonList* TYTHON_FN(sorted_by_tag)(TythonList* lst, int64_t lt_ops_handle) {
    auto* out = L(v(lst)->copy());
    TYTHON_FN(list_sort_by_tag)(out, lt_ops_handle);
    return out;
}

/* ── str_by_tag ──────────────────────────────────────────────────── */

TythonStr* TYTHON_FN(list_str_by_tag)(TythonList* list, int64_t elem_str_ops_handle) {
    std::string result = "[";
    auto* p = v(list);
    const TythonStrOps* ops = str_ops_from_handle(elem_str_ops_handle);
    for (int64_t i = 0; i < p->len; i++) {
        if (i > 0) result += ", ";
        TythonStr* elem_str = ops->str(p->data[i]);
        result.append(elem_str->data, static_cast<size_t>(elem_str->len));
    }
    result += "]";
    return TYTHON_FN(str_new)(result.c_str(), static_cast<int64_t>(result.size()));
}
