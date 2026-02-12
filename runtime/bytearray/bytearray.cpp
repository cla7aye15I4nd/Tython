#include "tython.h"
#include "internal/vec.h"

#include <cstdio>
#include <cstring>

using BAVec = tython::Vec<uint8_t>;

static_assert(sizeof(BAVec) == sizeof(TythonByteArray),
              "Vec<uint8_t> must be layout-compatible with TythonByteArray");

static auto* v(TythonByteArray* p)  { return reinterpret_cast<BAVec*>(p); }
static auto* BA(BAVec* p)           { return reinterpret_cast<TythonByteArray*>(p); }

/* ── construction ────────────────────────────────────────────────── */

TythonByteArray* TYTHON_FN(bytearray_new)(const uint8_t* data, int64_t len) {
    return BA(BAVec::create(data, len));
}

TythonByteArray* TYTHON_FN(bytearray_empty)(void) {
    return BA(BAVec::empty());
}

TythonByteArray* TYTHON_FN(bytearray_from_int)(int64_t n) {
    if (n < 0) {
        std::fprintf(stderr, "ValueError: negative count\n");
        std::exit(1);
    }
    return BA(BAVec::zero_filled(n));
}

TythonByteArray* TYTHON_FN(bytearray_from_bytes)(TythonBytes* b) {
    return BA(BAVec::create(b->data, b->len));
}

/* ── operations (delegated to Vec<uint8_t>) ──────────────────────── */

TythonByteArray* TYTHON_FN(bytearray_concat)(TythonByteArray* a, TythonByteArray* b) {
    return BA(v(a)->concat(v(b)));
}

TythonByteArray* TYTHON_FN(bytearray_repeat)(TythonByteArray* s, int64_t n) {
    return BA(v(s)->repeat(n));
}

int64_t TYTHON_FN(bytearray_len)(TythonByteArray* ba) { return v(ba)->len; }
int64_t TYTHON_FN(bytearray_cmp)(TythonByteArray* a, TythonByteArray* b) { return v(a)->cmp(v(b)); }
int64_t TYTHON_FN(bytearray_eq)(TythonByteArray* a, TythonByteArray* b) { return v(a)->eq(v(b)); }

void TYTHON_FN(print_bytearray)(TythonByteArray* ba) {
    std::printf("bytearray(");
    print_bytes_repr(v(ba)->data, v(ba)->len);
    std::printf(")");
}

TythonStr* TYTHON_FN(str_from_bytearray)(TythonByteArray* ba) {
    int64_t body_len = bytes_repr_body_len(v(ba)->data, v(ba)->len);
    /* bytearray(b'...')  →  "bytearray(b'" + body + "')" = 12 + body + 2 */
    int64_t total = 14 + body_len;
    auto* s = reinterpret_cast<TythonStr*>(
        __tython_malloc(static_cast<int64_t>(sizeof(TythonStr)) + total));
    s->len = total;
    char* p = s->data;
    std::memcpy(p, "bytearray(b'", 12); p += 12;
    p = bytes_repr_body_write(p, v(ba)->data, v(ba)->len);
    *p++ = '\''; *p = ')';
    return s;
}

void TYTHON_FN(bytearray_append)(TythonByteArray* ba, int64_t byte_val) {
    v(ba)->push(static_cast<uint8_t>(byte_val & 0xFF));
}

void TYTHON_FN(bytearray_extend)(TythonByteArray* ba, TythonBytes* other) {
    v(ba)->extend_from(other->data, other->len);
}

void TYTHON_FN(bytearray_clear)(TythonByteArray* ba) { v(ba)->clear(); }

void TYTHON_FN(bytearray_insert)(TythonByteArray* ba, int64_t index, int64_t byte_val) {
    v(ba)->insert_at(index, static_cast<uint8_t>(byte_val & 0xFF));
}

void TYTHON_FN(bytearray_remove)(TythonByteArray* ba, int64_t byte_val) {
    if (!v(ba)->remove_first(static_cast<uint8_t>(byte_val & 0xFF))) {
        std::fprintf(stderr, "ValueError: value not found in bytearray\n");
        std::exit(1);
    }
}

void TYTHON_FN(bytearray_reverse)(TythonByteArray* ba) { v(ba)->reverse(); }
