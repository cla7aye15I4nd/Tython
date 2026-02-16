#include "tython.h"
#include "gc/gc.h"

#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <string>

/* ── Open-addressing hash set ────────────────────────────────────────
   Replaces the former linear-scan array with O(1) amortised lookups.
   Slots hold either a live value, EMPTY, or DELETED sentinel.
   Capacity is always a power-of-two so masking replaces modulo.
   ────────────────────────────────────────────────────────────────── */

static constexpr int64_t EMPTY   = INT64_MIN;
static constexpr int64_t DELETED = INT64_MIN + 1;

static inline bool is_live(int64_t v) { return v != EMPTY && v != DELETED; }

// splitmix64 finalizer — excellent distribution for pointers and ints
static inline uint64_t hash_val(int64_t v) {
    uint64_t h = static_cast<uint64_t>(v);
    h ^= h >> 30;
    h *= 0xbf58476d1ce4e5b9ULL;
    h ^= h >> 27;
    h *= 0x94d049bb133111ebULL;
    h ^= h >> 31;
    return h;
}

// Hash a value using intrinsic_hash when eq_tag != 0, raw value otherwise.
static inline uint64_t tagged_hash(int64_t value, int64_t eq_tag) {
    int64_t h = eq_tag ? TYTHON_FN(intrinsic_hash)(eq_tag, value) : value;
    return hash_val(h);
}

/* ── Internal helpers ────────────────────────────────────────────── */

static void fill_empty(int64_t* data, int64_t cap) {
    for (int64_t i = 0; i < cap; i++) data[i] = EMPTY;
}

// Rehash all live entries into a fresh table of size new_cap (power of 2).
// eq_tag == 0 means use raw value hash; nonzero means use intrinsic_hash.
static void rehash(TythonSet* s, int64_t new_cap, int64_t eq_tag) {
    int64_t* old_data = s->data;
    int64_t  old_cap  = s->capacity;

    auto* new_data = static_cast<int64_t*>(__tython_gc_malloc(new_cap * sizeof(int64_t)));
    fill_empty(new_data, new_cap);

    uint64_t mask = static_cast<uint64_t>(new_cap - 1);
    int64_t count = 0;
    for (int64_t i = 0; i < old_cap; i++) {
        if (is_live(old_data[i])) {
            uint64_t idx = tagged_hash(old_data[i], eq_tag) & mask;
            while (new_data[idx] != EMPTY) idx = (idx + 1) & mask;
            new_data[idx] = old_data[i];
            count++;
        }
    }

    __tython_gc_free(old_data);
    s->data     = new_data;
    s->capacity = new_cap;
    s->len      = count;
}

static inline void maybe_grow(TythonSet* s, int64_t eq_tag) {
    if (s->capacity == 0) {
        rehash(s, 16, eq_tag);
    } else if (s->len * 4 >= s->capacity * 3) {   // 75% load factor
        rehash(s, s->capacity * 2, eq_tag);
    }
}

/* ── Probe helpers ───────────────────────────────────────────────── */

// Returns slot index if found, -1 if not found.
static int64_t find_value(TythonSet* s, int64_t value) {
    if (s->capacity == 0) return -1;
    uint64_t mask = static_cast<uint64_t>(s->capacity - 1);
    uint64_t idx  = hash_val(value) & mask;
    for (int64_t i = 0; i < s->capacity; i++) {
        int64_t slot = s->data[idx];
        if (slot == EMPTY)  return -1;
        if (slot == value)  return static_cast<int64_t>(idx);
        idx = (idx + 1) & mask;
    }
    return -1;
}

// by_tag variant: uses intrinsic_hash for probing, intrinsic_eq for comparison.
static int64_t find_value_by_tag(TythonSet* s, int64_t value, int64_t eq_tag) {
    if (s->capacity == 0) return -1;
    uint64_t mask = static_cast<uint64_t>(s->capacity - 1);
    uint64_t idx  = tagged_hash(value, eq_tag) & mask;
    for (int64_t i = 0; i < s->capacity; i++) {
        int64_t slot = s->data[idx];
        if (slot == EMPTY) return -1;
        if (is_live(slot) && TYTHON_FN(intrinsic_eq)(eq_tag, slot, value) != 0)
            return static_cast<int64_t>(idx);
        idx = (idx + 1) & mask;
    }
    return -1;
}

/* ── Single-pass insert (combined find + insert) ─────────────────── */

static void insert_value(TythonSet* s, int64_t value) {
    maybe_grow(s, 0);
    uint64_t mask = static_cast<uint64_t>(s->capacity - 1);
    uint64_t idx  = hash_val(value) & mask;
    int64_t  insert_pos = -1;
    for (int64_t i = 0; i < s->capacity; i++) {
        int64_t slot = s->data[idx];
        if (slot == EMPTY) {
            int64_t p = insert_pos >= 0 ? insert_pos : static_cast<int64_t>(idx);
            s->data[p] = value;
            s->len++;
            return;
        }
        if (slot == DELETED) {
            if (insert_pos < 0) insert_pos = static_cast<int64_t>(idx);
        } else if (slot == value) {
            return;   // already present
        }
        idx = (idx + 1) & mask;
    }
    // Only reachable if table is full of live + deleted (shouldn't happen at 75% load)
    if (insert_pos >= 0) { s->data[insert_pos] = value; s->len++; }
}

static void insert_value_by_tag(TythonSet* s, int64_t value, int64_t eq_tag) {
    maybe_grow(s, eq_tag);
    uint64_t mask = static_cast<uint64_t>(s->capacity - 1);
    uint64_t idx  = tagged_hash(value, eq_tag) & mask;
    int64_t  insert_pos = -1;
    for (int64_t i = 0; i < s->capacity; i++) {
        int64_t slot = s->data[idx];
        if (slot == EMPTY) {
            int64_t p = insert_pos >= 0 ? insert_pos : static_cast<int64_t>(idx);
            s->data[p] = value;
            s->len++;
            return;
        }
        if (slot == DELETED) {
            if (insert_pos < 0) insert_pos = static_cast<int64_t>(idx);
        } else if (TYTHON_FN(intrinsic_eq)(eq_tag, slot, value) != 0) {
            return;   // already present
        }
        idx = (idx + 1) & mask;
    }
    if (insert_pos >= 0) { s->data[insert_pos] = value; s->len++; }
}

/* ── Delete helper ───────────────────────────────────────────────── */

static inline void delete_at(TythonSet* s, int64_t idx) {
    s->data[idx] = DELETED;
    s->len--;
}

/* ── Public API ──────────────────────────────────────────────────── */

TythonSet* TYTHON_FN(set_empty)(void) {
    auto* s = static_cast<TythonSet*>(__tython_gc_malloc(sizeof(TythonSet)));
    s->len      = 0;
    s->capacity = 0;
    s->data     = nullptr;
    return s;
}

int64_t TYTHON_FN(set_len)(TythonSet* s) { return s->len; }

int64_t TYTHON_FN(set_contains)(TythonSet* s, int64_t value) {
    return find_value(s, value) >= 0;
}

int64_t TYTHON_FN(set_contains_by_tag)(TythonSet* s, int64_t value, int64_t eq_tag) {
    return find_value_by_tag(s, value, eq_tag) >= 0;
}

void TYTHON_FN(set_add)(TythonSet* s, int64_t value) { insert_value(s, value); }

void TYTHON_FN(set_add_by_tag)(TythonSet* s, int64_t value, int64_t eq_tag) {
    insert_value_by_tag(s, value, eq_tag);
}

void TYTHON_FN(set_remove)(TythonSet* s, int64_t value) {
    int64_t idx = find_value(s, value);
    if (idx < 0) {
        TYTHON_FN(raise)(TYTHON_EXC_KEY_ERROR, TYTHON_FN(str_new)("value not found", 15));
        __builtin_unreachable();
    }
    delete_at(s, idx);
}

void TYTHON_FN(set_remove_by_tag)(TythonSet* s, int64_t value, int64_t eq_tag) {
    int64_t idx = find_value_by_tag(s, value, eq_tag);
    if (idx < 0) {
        TYTHON_FN(raise)(TYTHON_EXC_KEY_ERROR, TYTHON_FN(str_new)("value not found", 15));
        __builtin_unreachable();
    }
    delete_at(s, idx);
}

void TYTHON_FN(set_discard)(TythonSet* s, int64_t value) {
    int64_t idx = find_value(s, value);
    if (idx >= 0) delete_at(s, idx);
}

void TYTHON_FN(set_discard_by_tag)(TythonSet* s, int64_t value, int64_t eq_tag) {
    int64_t idx = find_value_by_tag(s, value, eq_tag);
    if (idx >= 0) delete_at(s, idx);
}

/* ── Bulk set-algebra operations ─────────────────────────────────── */

TythonSet* TYTHON_FN(set_union_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_tag) {
    auto* out = TYTHON_FN(set_copy)(a);
    for (int64_t i = 0; i < b->capacity; i++)
        if (is_live(b->data[i]))
            TYTHON_FN(set_add_by_tag)(out, b->data[i], eq_tag);
    return out;
}

void TYTHON_FN(set_update_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_tag) {
    for (int64_t i = 0; i < b->capacity; i++)
        if (is_live(b->data[i]))
            TYTHON_FN(set_add_by_tag)(a, b->data[i], eq_tag);
}

TythonSet* TYTHON_FN(set_intersection_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_tag) {
    auto* out = TYTHON_FN(set_empty)();
    for (int64_t i = 0; i < a->capacity; i++)
        if (is_live(a->data[i]) && find_value_by_tag(b, a->data[i], eq_tag) >= 0)
            TYTHON_FN(set_add_by_tag)(out, a->data[i], eq_tag);
    return out;
}

void TYTHON_FN(set_intersection_update_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_tag) {
    // Collect entries to keep, then rebuild
    for (int64_t i = 0; i < a->capacity; i++)
        if (is_live(a->data[i]) && find_value_by_tag(b, a->data[i], eq_tag) < 0) {
            a->data[i] = DELETED;
            a->len--;
        }
}

TythonSet* TYTHON_FN(set_difference_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_tag) {
    auto* out = TYTHON_FN(set_empty)();
    for (int64_t i = 0; i < a->capacity; i++)
        if (is_live(a->data[i]) && find_value_by_tag(b, a->data[i], eq_tag) < 0)
            TYTHON_FN(set_add_by_tag)(out, a->data[i], eq_tag);
    return out;
}

void TYTHON_FN(set_difference_update_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_tag) {
    for (int64_t i = 0; i < a->capacity; i++)
        if (is_live(a->data[i]) && find_value_by_tag(b, a->data[i], eq_tag) >= 0) {
            a->data[i] = DELETED;
            a->len--;
        }
}

TythonSet* TYTHON_FN(set_symmetric_difference_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_tag) {
    auto* out = TYTHON_FN(set_empty)();
    for (int64_t i = 0; i < a->capacity; i++)
        if (is_live(a->data[i]) && find_value_by_tag(b, a->data[i], eq_tag) < 0)
            TYTHON_FN(set_add_by_tag)(out, a->data[i], eq_tag);
    for (int64_t i = 0; i < b->capacity; i++)
        if (is_live(b->data[i]) && find_value_by_tag(a, b->data[i], eq_tag) < 0)
            TYTHON_FN(set_add_by_tag)(out, b->data[i], eq_tag);
    return out;
}

void TYTHON_FN(set_symmetric_difference_update_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_tag) {
    auto* tmp = TYTHON_FN(set_symmetric_difference_by_tag)(a, b, eq_tag);
    // Replace a's contents with tmp
    a->data     = tmp->data;
    a->capacity = tmp->capacity;
    a->len      = tmp->len;
}

/* ── Relational / subset operations ──────────────────────────────── */

int64_t TYTHON_FN(set_isdisjoint_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_tag) {
    TythonSet* smaller = a->len <= b->len ? a : b;
    TythonSet* larger  = a->len <= b->len ? b : a;
    for (int64_t i = 0; i < smaller->capacity; i++)
        if (is_live(smaller->data[i]) && find_value_by_tag(larger, smaller->data[i], eq_tag) >= 0)
            return 0;
    return 1;
}

int64_t TYTHON_FN(set_issubset_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_tag) {
    if (a->len > b->len) return 0;
    for (int64_t i = 0; i < a->capacity; i++)
        if (is_live(a->data[i]) && find_value_by_tag(b, a->data[i], eq_tag) < 0)
            return 0;
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

/* ── Augmented assignment operators ──────────────────────────────── */

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

/* ── Misc ────────────────────────────────────────────────────────── */

int64_t TYTHON_FN(set_pop)(TythonSet* s) {
    if (s->len == 0) {
        TYTHON_FN(raise)(TYTHON_EXC_KEY_ERROR, TYTHON_FN(str_new)("pop from empty set", 18));
        __builtin_unreachable();
    }
    for (int64_t i = 0; i < s->capacity; i++) {
        if (is_live(s->data[i])) {
            int64_t out = s->data[i];
            delete_at(s, i);
            return out;
        }
    }
    __builtin_unreachable();
}

void TYTHON_FN(set_clear)(TythonSet* s) {
    fill_empty(s->data, s->capacity);
    s->len = 0;
}

int64_t TYTHON_FN(set_eq)(TythonSet* a, TythonSet* b) {
    if (a == b) return 1;
    if (a->len != b->len) return 0;
    for (int64_t i = 0; i < a->capacity; i++)
        if (is_live(a->data[i]) && find_value(b, a->data[i]) < 0)
            return 0;
    return 1;
}

int64_t TYTHON_FN(set_eq_by_tag)(TythonSet* a, TythonSet* b, int64_t eq_tag) {
    if (a == b) return 1;
    if (a->len != b->len) return 0;
    for (int64_t i = 0; i < a->capacity; i++)
        if (is_live(a->data[i]) && find_value_by_tag(b, a->data[i], eq_tag) < 0)
            return 0;
    return 1;
}

/* ── str_by_tag ──────────────────────────────────────────────────── */

TythonStr* TYTHON_FN(set_str_by_tag)(TythonSet* set, int64_t elem_str_tag) {
    std::string result = "{";
    bool first = true;
    for (int64_t i = 0; i < set->capacity; i++) {
        if (!is_live(set->data[i])) continue;
        if (!first) result += ", ";
        first = false;
        TythonStr* elem_str = TYTHON_FN(intrinsic_str)(elem_str_tag, reinterpret_cast<void*>(set->data[i]));
        result.append(elem_str->data, static_cast<size_t>(elem_str->len));
    }
    result += "}";
    return TYTHON_FN(str_new)(result.c_str(), static_cast<int64_t>(result.size()));
}

TythonSet* TYTHON_FN(set_copy)(TythonSet* s) {
    auto* out = static_cast<TythonSet*>(__tython_gc_malloc(sizeof(TythonSet)));
    out->len      = s->len;
    out->capacity = s->capacity;
    if (s->capacity > 0) {
        out->data = static_cast<int64_t*>(__tython_gc_malloc(s->capacity * sizeof(int64_t)));
        std::memcpy(out->data, s->data, static_cast<size_t>(s->capacity) * sizeof(int64_t));
    } else {
        out->data = nullptr;
    }
    return out;
}
