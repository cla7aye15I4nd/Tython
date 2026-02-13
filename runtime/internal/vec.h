#ifndef TYTHON_INTERNAL_VEC_H
#define TYTHON_INTERNAL_VEC_H

#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <algorithm>

#include "../gc/gc.h"

// Use regular GC allocation for vectors (can contain pointers)
#define __tython_malloc __tython_gc_malloc

namespace tython {

/* ── Vec<T> ─────────────────────────────────────────────────────────
   Growable array template.  Layout-compatible with both TythonList
   (T = int64_t) and TythonByteArray (T = uint8_t).
   ────────────────────────────────────────────────────────────────── */
template<typename T>
struct Vec {
    int64_t len;
    int64_t capacity;
    T* data;

    /* ── construction ────────────────────────────────────────────── */

    static Vec* create(const T* src, int64_t n) {
        auto* v = static_cast<Vec*>(__tython_malloc(sizeof(Vec)));
        int64_t cap = n > 8 ? n : 8;
        v->len = n;
        v->capacity = cap;
        v->data = static_cast<T*>(__tython_malloc(cap * static_cast<int64_t>(sizeof(T))));
        if (n > 0 && src)
            std::memcpy(v->data, src, static_cast<size_t>(n) * sizeof(T));
        return v;
    }

    static Vec* empty() { return create(nullptr, 0); }

    static Vec* zero_filled(int64_t n) {
        auto* v = create(nullptr, n);
        std::memset(v->data, 0, static_cast<size_t>(n) * sizeof(T));
        return v;
    }

    /* ── growth ──────────────────────────────────────────────────── */

    void grow(int64_t min_cap) {
        if (min_cap <= capacity) return;
        int64_t new_cap = capacity * 2;
        if (new_cap < min_cap) new_cap = min_cap;
        if (new_cap < 8) new_cap = 8;
        auto* new_data = static_cast<T*>(
            __tython_malloc(new_cap * static_cast<int64_t>(sizeof(T))));
        std::memcpy(new_data, data, static_cast<size_t>(len) * sizeof(T));
        __tython_gc_free(data);
        data = new_data;
        capacity = new_cap;
    }

    /* ── element operations ──────────────────────────────────────── */

    void push(T value) {
        grow(len + 1);
        data[len++] = value;
    }

    T pop_back() { return data[--len]; }

    void clear() { len = 0; }

    void insert_at(int64_t index, T value) {
        int64_t idx = index;
        if (idx < 0) idx += len;
        if (idx < 0) idx = 0;
        if (idx > len) idx = len;
        grow(len + 1);
        std::memmove(&data[idx + 1], &data[idx],
                     static_cast<size_t>(len - idx) * sizeof(T));
        data[idx] = value;
        len++;
    }

    bool remove_first(T value) {
        auto* it = std::find(data, data + len, value);
        if (it == data + len) return false;
        std::memmove(it, it + 1,
                     static_cast<size_t>((data + len) - (it + 1)) * sizeof(T));
        len--;
        return true;
    }

    void reverse() { std::reverse(data, data + len); }

    /* ── queries ─────────────────────────────────────────────────── */

    int64_t contains(T value) const {
        return std::find(data, data + len, value) != data + len ? 1 : 0;
    }

    int64_t index_of(T value) const {
        auto* it = std::find(data, data + len, value);
        return it != data + len ? static_cast<int64_t>(it - data) : -1;
    }

    int64_t count_of(T value) const {
        return static_cast<int64_t>(std::count(data, data + len, value));
    }

    /* ── bulk operations ─────────────────────────────────────────── */

    void extend_from(const T* src, int64_t n) {
        grow(len + n);
        std::memcpy(&data[len], src, static_cast<size_t>(n) * sizeof(T));
        len += n;
    }

    Vec* copy() const { return create(data, len); }

    Vec* concat(const Vec* other) const {
        int64_t new_len = len + other->len;
        int64_t cap = new_len > 8 ? new_len : 8;
        auto* r = static_cast<Vec*>(__tython_malloc(sizeof(Vec)));
        r->len = new_len;
        r->capacity = cap;
        r->data = static_cast<T*>(
            __tython_malloc(cap * static_cast<int64_t>(sizeof(T))));
        std::memcpy(r->data, data, static_cast<size_t>(len) * sizeof(T));
        std::memcpy(r->data + len, other->data,
                     static_cast<size_t>(other->len) * sizeof(T));
        return r;
    }

    Vec* repeat(int64_t n) const {
        if (n <= 0) return empty();
        int64_t new_len = len * n;
        auto* r = static_cast<Vec*>(__tython_malloc(sizeof(Vec)));
        r->len = new_len;
        r->capacity = new_len;
        r->data = static_cast<T*>(
            __tython_malloc(new_len * static_cast<int64_t>(sizeof(T))));
        for (int64_t i = 0; i < n; i++)
            std::memcpy(r->data + i * len, data,
                         static_cast<size_t>(len) * sizeof(T));
        return r;
    }

    /* ── comparison ──────────────────────────────────────────────── */

    int64_t cmp(const Vec* other) const {
        int64_t min_len = len < other->len ? len : other->len;
        int c = std::memcmp(data, other->data,
                            static_cast<size_t>(min_len) * sizeof(T));
        if (c != 0) return c < 0 ? -1 : 1;
        if (len < other->len) return -1;
        if (len > other->len) return 1;
        return 0;
    }

    int64_t eq(const Vec* other) const {
        if (this == other) return 1;
        if (len != other->len) return 0;
        return std::memcmp(data, other->data,
                           static_cast<size_t>(len) * sizeof(T)) == 0 ? 1 : 0;
    }

    /* ── sorting ─────────────────────────────────────────────────── */

    void sort() { std::sort(data, data + len); }

    template<typename Compare>
    void sort(Compare comp) { std::sort(data, data + len, comp); }
};

} // namespace tython

#endif /* TYTHON_INTERNAL_VEC_H */
