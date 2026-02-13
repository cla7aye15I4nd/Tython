#ifndef TYTHON_INTERNAL_BUF_H
#define TYTHON_INTERNAL_BUF_H

#include <cstdint>
#include <cstdlib>
#include <cstring>

#include "../gc/gc.h"

// Use atomic allocation for buffers (strings/bytes contain no pointers)
#define __tython_malloc __tython_gc_malloc_atomic

namespace tython {

/* ── Buf<T> ─────────────────────────────────────────────────────────
   Immutable buffer with flexible array member.  Data is stored inline
   right after the header (single allocation, better cache locality).

   Layout-compatible with TythonStr (T = char) and TythonBytes
   (T = uint8_t).
   ────────────────────────────────────────────────────────────────── */
template<typename T>
struct Buf {
    int64_t len;
    T data[]; /* flexible array member */

    /* ── allocation helper ───────────────────────────────────────── */

    static int64_t alloc_size(int64_t n) {
        return static_cast<int64_t>(
            sizeof(Buf) + static_cast<size_t>(n > 0 ? n : 1) * sizeof(T));
    }

    /* ── construction ────────────────────────────────────────────── */

    static Buf* create(const T* src, int64_t n) {
        auto* b = static_cast<Buf*>(__tython_malloc(alloc_size(n)));
        b->len = n;
        if (n > 0 && src)
            std::memcpy(b->data, src, static_cast<size_t>(n) * sizeof(T));
        return b;
    }

    /* ── operations ──────────────────────────────────────────────── */

    Buf* concat(const Buf* other) const {
        int64_t new_len = len + other->len;
        auto* r = static_cast<Buf*>(__tython_malloc(alloc_size(new_len)));
        r->len = new_len;
        std::memcpy(r->data, data, static_cast<size_t>(len) * sizeof(T));
        std::memcpy(r->data + len, other->data,
                     static_cast<size_t>(other->len) * sizeof(T));
        return r;
    }

    Buf* repeat(int64_t n) const {
        if (n <= 0) return create(nullptr, 0);
        int64_t new_len = len * n;
        auto* r = static_cast<Buf*>(__tython_malloc(alloc_size(new_len)));
        r->len = new_len;
        for (int64_t i = 0; i < n; i++)
            std::memcpy(r->data + i * len, data,
                         static_cast<size_t>(len) * sizeof(T));
        return r;
    }

    /* ── comparison ──────────────────────────────────────────────── */

    int64_t cmp(const Buf* other) const {
        int64_t min_len = len < other->len ? len : other->len;
        int c = std::memcmp(data, other->data,
                            static_cast<size_t>(min_len) * sizeof(T));
        if (c != 0) return c < 0 ? -1 : 1;
        if (len < other->len) return -1;
        if (len > other->len) return 1;
        return 0;
    }

    int64_t eq(const Buf* other) const {
        if (len != other->len) return 0;
        return std::memcmp(data, other->data,
                           static_cast<size_t>(len) * sizeof(T)) == 0 ? 1 : 0;
    }

    int64_t contains_sub(const Buf* needle) const {
        if (needle->len == 0) return 1;
        if (needle->len > len) return 0;
        for (int64_t i = 0; i <= len - needle->len; i++) {
            if (std::memcmp(data + i, needle->data,
                            static_cast<size_t>(needle->len) * sizeof(T)) == 0)
                return 1;
        }
        return 0;
    }
};

} // namespace tython

#endif /* TYTHON_INTERNAL_BUF_H */
