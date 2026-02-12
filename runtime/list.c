#include "tython.h"

TythonList* __tython_list_new(const int64_t* data, int64_t len) {
    TythonList* lst = (TythonList*)__tython_malloc(sizeof(TythonList));
    int64_t cap = len > 8 ? len : 8;
    lst->len = len;
    lst->capacity = cap;
    lst->data = (int64_t*)__tython_malloc(cap * (int64_t)sizeof(int64_t));
    if (len > 0 && data) {
        memcpy(lst->data, data, (size_t)len * sizeof(int64_t));
    }
    return lst;
}

TythonList* __tython_list_empty(void) {
    return __tython_list_new(NULL, 0);
}

int64_t __tython_list_len(TythonList* lst) {
    return lst->len;
}

static int64_t resolve_index(TythonList* lst, int64_t index) {
    int64_t resolved = index;
    if (resolved < 0) resolved += lst->len;
    if (resolved < 0 || resolved >= lst->len) {
        fprintf(stderr, "IndexError: list index out of range\n");
        exit(1);
    }
    return resolved;
}

int64_t __tython_list_get(TythonList* lst, int64_t index) {
    return lst->data[resolve_index(lst, index)];
}

void __tython_list_set(TythonList* lst, int64_t index, int64_t value) {
    lst->data[resolve_index(lst, index)] = value;
}

void __tython_list_append(TythonList* lst, int64_t value) {
    if (lst->len >= lst->capacity) {
        int64_t new_cap = lst->capacity * 2;
        if (new_cap < 8) new_cap = 8;
        int64_t* new_data = (int64_t*)__tython_malloc(new_cap * (int64_t)sizeof(int64_t));
        memcpy(new_data, lst->data, (size_t)lst->len * sizeof(int64_t));
        free(lst->data);
        lst->data = new_data;
        lst->capacity = new_cap;
    }
    lst->data[lst->len] = value;
    lst->len++;
}

int64_t __tython_list_pop(TythonList* lst) {
    if (lst->len == 0) {
        fprintf(stderr, "IndexError: pop from empty list\n");
        exit(1);
    }
    lst->len--;
    return lst->data[lst->len];
}

void __tython_list_clear(TythonList* lst) {
    lst->len = 0;
}

/* ── containment ─────────────────────────────────────────────────── */

int64_t __tython_list_contains(TythonList* lst, int64_t value) {
    for (int64_t i = 0; i < lst->len; i++) {
        if (lst->data[i] == value) return 1;
    }
    return 0;
}

/* ── insert / remove / index / reverse / sort / extend / copy / count ── */

void __tython_list_insert(TythonList* lst, int64_t index, int64_t value) {
    /* Resolve index (clamp, don't error) */
    int64_t idx = index;
    if (idx < 0) idx += lst->len;
    if (idx < 0) idx = 0;
    if (idx > lst->len) idx = lst->len;
    /* Grow if needed */
    if (lst->len >= lst->capacity) {
        int64_t new_cap = lst->capacity * 2;
        if (new_cap < 8) new_cap = 8;
        int64_t* new_data = (int64_t*)__tython_malloc(new_cap * (int64_t)sizeof(int64_t));
        memcpy(new_data, lst->data, (size_t)lst->len * sizeof(int64_t));
        free(lst->data);
        lst->data = new_data;
        lst->capacity = new_cap;
    }
    /* Shift right */
    memmove(&lst->data[idx + 1], &lst->data[idx],
            (size_t)(lst->len - idx) * sizeof(int64_t));
    lst->data[idx] = value;
    lst->len++;
}

void __tython_list_remove(TythonList* lst, int64_t value) {
    for (int64_t i = 0; i < lst->len; i++) {
        if (lst->data[i] == value) {
            memmove(&lst->data[i], &lst->data[i + 1],
                    (size_t)(lst->len - i - 1) * sizeof(int64_t));
            lst->len--;
            return;
        }
    }
    __tython_raise(TYTHON_EXC_VALUE_ERROR, __tython_str_new("list.remove(x): x not in list", 30));
    __builtin_unreachable();
}

int64_t __tython_list_index(TythonList* lst, int64_t value) {
    for (int64_t i = 0; i < lst->len; i++) {
        if (lst->data[i] == value) return i;
    }
    __tython_raise(TYTHON_EXC_VALUE_ERROR, __tython_str_new("x not in list", 13));
    __builtin_unreachable();
}

int64_t __tython_list_count(TythonList* lst, int64_t value) {
    int64_t count = 0;
    for (int64_t i = 0; i < lst->len; i++) {
        if (lst->data[i] == value) count++;
    }
    return count;
}

void __tython_list_reverse(TythonList* lst) {
    for (int64_t i = 0, j = lst->len - 1; i < j; i++, j--) {
        int64_t tmp = lst->data[i];
        lst->data[i] = lst->data[j];
        lst->data[j] = tmp;
    }
}

static int cmp_int_asc(const void* a, const void* b) {
    int64_t va = *(const int64_t*)a;
    int64_t vb = *(const int64_t*)b;
    return (va > vb) - (va < vb);
}

static int cmp_float_asc(const void* a, const void* b) {
    double va, vb;
    memcpy(&va, a, sizeof(double));
    memcpy(&vb, b, sizeof(double));
    return (va > vb) - (va < vb);
}

void __tython_list_sort_int(TythonList* lst) {
    qsort(lst->data, (size_t)lst->len, sizeof(int64_t), cmp_int_asc);
}

void __tython_list_sort_float(TythonList* lst) {
    qsort(lst->data, (size_t)lst->len, sizeof(int64_t), cmp_float_asc);
}

TythonList* __tython_sorted_int(TythonList* lst) {
    TythonList* out = __tython_list_copy(lst);
    __tython_list_sort_int(out);
    return out;
}

TythonList* __tython_sorted_float(TythonList* lst) {
    TythonList* out = __tython_list_copy(lst);
    __tython_list_sort_float(out);
    return out;
}

void __tython_list_extend(TythonList* lst, TythonList* other) {
    int64_t new_len = lst->len + other->len;
    if (new_len > lst->capacity) {
        int64_t new_cap = new_len * 2;
        int64_t* new_data = (int64_t*)__tython_malloc(new_cap * (int64_t)sizeof(int64_t));
        memcpy(new_data, lst->data, (size_t)lst->len * sizeof(int64_t));
        free(lst->data);
        lst->data = new_data;
        lst->capacity = new_cap;
    }
    memcpy(&lst->data[lst->len], other->data, (size_t)other->len * sizeof(int64_t));
    lst->len = new_len;
}

TythonList* __tython_list_copy(TythonList* lst) {
    return __tython_list_new(lst->data, lst->len);
}

/* ── aggregate builtins ──────────────────────────────────────────── */

int64_t __tython_sum_int(TythonList* lst) {
    int64_t sum = 0;
    for (int64_t i = 0; i < lst->len; i++) sum += lst->data[i];
    return sum;
}

double __tython_sum_float(TythonList* lst) {
    double sum = 0.0;
    for (int64_t i = 0; i < lst->len; i++) {
        double v;
        memcpy(&v, &lst->data[i], sizeof(double));
        sum += v;
    }
    return sum;
}

int64_t __tython_sum_int_start(TythonList* lst, int64_t start) {
    int64_t sum = start;
    for (int64_t i = 0; i < lst->len; i++) sum += lst->data[i];
    return sum;
}

double __tython_sum_float_start(TythonList* lst, double start) {
    double sum = start;
    for (int64_t i = 0; i < lst->len; i++) {
        double v;
        memcpy(&v, &lst->data[i], sizeof(double));
        sum += v;
    }
    return sum;
}

int64_t __tython_all_list(TythonList* lst) {
    for (int64_t i = 0; i < lst->len; i++) {
        if (lst->data[i] == 0) return 0;
    }
    return 1;
}

int64_t __tython_any_list(TythonList* lst) {
    for (int64_t i = 0; i < lst->len; i++) {
        if (lst->data[i] != 0) return 1;
    }
    return 0;
}

/* ── equality ────────────────────────────────────────────────────── */

int64_t __tython_list_eq_shallow(TythonList* a, TythonList* b) {
    if (a == b) return 1;
    if (a->len != b->len) return 0;
    for (int64_t i = 0; i < a->len; i++) {
        if (a->data[i] != b->data[i]) return 0;
    }
    return 1;
}

int64_t __tython_list_eq_deep(TythonList* a, TythonList* b, int64_t depth) {
    if (a == b) return 1;
    if (a->len != b->len) return 0;
    if (depth <= 0) {
        /* leaf level: bitwise compare */
        for (int64_t i = 0; i < a->len; i++) {
            if (a->data[i] != b->data[i]) return 0;
        }
    } else {
        /* recurse into nested lists */
        for (int64_t i = 0; i < a->len; i++) {
            TythonList* ai = (TythonList*)(uintptr_t)a->data[i];
            TythonList* bi = (TythonList*)(uintptr_t)b->data[i];
            if (!__tython_list_eq_deep(ai, bi, depth - 1)) return 0;
        }
    }
    return 1;
}

/* ── print helpers ────────────────────────────────────────────────── */

void __tython_print_list_int(TythonList* lst) {
    printf("[");
    for (int64_t i = 0; i < lst->len; i++) {
        if (i > 0) printf(", ");
        printf("%ld", (long)lst->data[i]);
    }
    printf("]");
}

void __tython_print_list_float(TythonList* lst) {
    printf("[");
    for (int64_t i = 0; i < lst->len; i++) {
        if (i > 0) printf(", ");
        double val;
        memcpy(&val, &lst->data[i], sizeof(double));
        /* Match Python: if the value is integral, print with .0 */
        if (val == (double)(int64_t)val && val >= -1e15 && val <= 1e15) {
            printf("%.1f", val);
        } else {
            printf("%g", val);
        }
    }
    printf("]");
}

void __tython_print_list_bool(TythonList* lst) {
    printf("[");
    for (int64_t i = 0; i < lst->len; i++) {
        if (i > 0) printf(", ");
        printf("%s", lst->data[i] ? "True" : "False");
    }
    printf("]");
}

void __tython_print_list_str(TythonList* lst) {
    printf("[");
    for (int64_t i = 0; i < lst->len; i++) {
        if (i > 0) printf(", ");
        TythonStr* s = (TythonStr*)(uintptr_t)lst->data[i];
        printf("'%.*s'", (int)s->len, s->data);
    }
    printf("]");
}

void __tython_print_list_bytes(TythonList* lst) {
    printf("[");
    for (int64_t i = 0; i < lst->len; i++) {
        if (i > 0) printf(", ");
        TythonBytes* b = (TythonBytes*)(uintptr_t)lst->data[i];
        print_bytes_repr(b->data, b->len);
    }
    printf("]");
}

void __tython_print_list_bytearray(TythonList* lst) {
    printf("[");
    for (int64_t i = 0; i < lst->len; i++) {
        if (i > 0) printf(", ");
        TythonByteArray* ba = (TythonByteArray*)(uintptr_t)lst->data[i];
        printf("bytearray(");
        print_bytes_repr(ba->data, ba->len);
        printf(")");
    }
    printf("]");
}
