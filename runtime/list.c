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
