#include "tython.h"
#include "internal/vec.h"

#include <cstdio>
#include <cstring>

using BAVec = tython::Vec<uint8_t>;

static_assert(sizeof(BAVec) == sizeof(TythonByteArray),
              "Vec<uint8_t> must be layout-compatible with TythonByteArray");

static auto* v(TythonByteArray* p)  { return reinterpret_cast<BAVec*>(p); }
static auto* BA(BAVec* p)           { return reinterpret_cast<TythonByteArray*>(p); }

struct BytesTriple {
    TythonBytes* a;
    TythonBytes* b;
    TythonBytes* c;
};

struct ByteArrayTriple {
    TythonByteArray* a;
    TythonByteArray* b;
    TythonByteArray* c;
};

static TythonBytes* ba_as_bytes(TythonByteArray* ba) {
    return TYTHON_FN(bytes_new)(v(ba)->data, v(ba)->len);
}

static TythonByteArray* ba_from_bytes(TythonBytes* b) {
    return TYTHON_FN(bytearray_from_bytes)(b);
}

static void* list_bytes_to_bytearray(void* list_ptr) {
    auto* bytes_list = static_cast<TythonList*>(list_ptr);
    auto* out = TYTHON_FN(list_empty)();
    for (int64_t i = 0; i < bytes_list->len; i++) {
        auto* item = reinterpret_cast<TythonBytes*>(static_cast<uintptr_t>(bytes_list->data[i]));
        auto* out_item = ba_from_bytes(item);
        TYTHON_FN(list_append)(out, static_cast<int64_t>(reinterpret_cast<uintptr_t>(out_item)));
    }
    return out;
}

static void* tuple_bytes_to_bytearray(void* tuple_ptr) {
    auto* in = static_cast<BytesTriple*>(tuple_ptr);
    auto* out = static_cast<ByteArrayTriple*>(__tython_malloc(static_cast<int64_t>(sizeof(ByteArrayTriple))));
    out->a = ba_from_bytes(in->a);
    out->b = ba_from_bytes(in->b);
    out->c = ba_from_bytes(in->c);
    return out;
}

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

TythonByteArray* TYTHON_FN(bytearray_copy)(TythonByteArray* ba) {
    return BA(v(ba)->copy());
}

int64_t TYTHON_FN(bytearray_pop)(TythonByteArray* ba) {
    if (v(ba)->len == 0) {
        TYTHON_FN(raise)(TYTHON_EXC_INDEX_ERROR, TYTHON_FN(str_new)("pop from empty bytearray", 24));
        __builtin_unreachable();
    }
    return static_cast<int64_t>(v(ba)->pop_back());
}

TythonByteArray* TYTHON_FN(bytearray_capitalize)(TythonByteArray* ba) {
    return ba_from_bytes(TYTHON_FN(bytes_capitalize)(ba_as_bytes(ba)));
}

TythonByteArray* TYTHON_FN(bytearray_center)(TythonByteArray* ba, int64_t width, TythonBytes* fill) {
    return ba_from_bytes(TYTHON_FN(bytes_center)(ba_as_bytes(ba), width, fill));
}

int64_t TYTHON_FN(bytearray_count)(TythonByteArray* ba, TythonBytes* sub) {
    return TYTHON_FN(bytes_count)(ba_as_bytes(ba), sub);
}

TythonStr* TYTHON_FN(bytearray_decode)(TythonByteArray* ba) {
    return TYTHON_FN(bytes_decode)(ba_as_bytes(ba));
}

int64_t TYTHON_FN(bytearray_endswith)(TythonByteArray* ba, TythonBytes* suffix) {
    return TYTHON_FN(bytes_endswith)(ba_as_bytes(ba), suffix);
}

TythonByteArray* TYTHON_FN(bytearray_expandtabs)(TythonByteArray* ba, int64_t tabsize) {
    return ba_from_bytes(TYTHON_FN(bytes_expandtabs)(ba_as_bytes(ba), tabsize));
}

int64_t TYTHON_FN(bytearray_find)(TythonByteArray* ba, TythonBytes* sub) {
    return TYTHON_FN(bytes_find)(ba_as_bytes(ba), sub);
}

TythonByteArray* TYTHON_FN(bytearray_fromhex)(TythonByteArray* self, TythonStr* hex) {
    return ba_from_bytes(TYTHON_FN(bytes_fromhex)(ba_as_bytes(self), hex));
}

TythonStr* TYTHON_FN(bytearray_hex)(TythonByteArray* ba) {
    return TYTHON_FN(bytes_hex)(ba_as_bytes(ba));
}

int64_t TYTHON_FN(bytearray_index)(TythonByteArray* ba, TythonBytes* sub) {
    return TYTHON_FN(bytes_index)(ba_as_bytes(ba), sub);
}

int64_t TYTHON_FN(bytearray_isalnum)(TythonByteArray* ba) {
    return TYTHON_FN(bytes_isalnum)(ba_as_bytes(ba));
}

int64_t TYTHON_FN(bytearray_isalpha)(TythonByteArray* ba) {
    return TYTHON_FN(bytes_isalpha)(ba_as_bytes(ba));
}

int64_t TYTHON_FN(bytearray_isascii)(TythonByteArray* ba) {
    return TYTHON_FN(bytes_isascii)(ba_as_bytes(ba));
}

int64_t TYTHON_FN(bytearray_isdigit)(TythonByteArray* ba) {
    return TYTHON_FN(bytes_isdigit)(ba_as_bytes(ba));
}

int64_t TYTHON_FN(bytearray_islower)(TythonByteArray* ba) {
    return TYTHON_FN(bytes_islower)(ba_as_bytes(ba));
}

int64_t TYTHON_FN(bytearray_isspace)(TythonByteArray* ba) {
    return TYTHON_FN(bytes_isspace)(ba_as_bytes(ba));
}

int64_t TYTHON_FN(bytearray_istitle)(TythonByteArray* ba) {
    return TYTHON_FN(bytes_istitle)(ba_as_bytes(ba));
}

int64_t TYTHON_FN(bytearray_isupper)(TythonByteArray* ba) {
    return TYTHON_FN(bytes_isupper)(ba_as_bytes(ba));
}

TythonByteArray* TYTHON_FN(bytearray_join)(TythonByteArray* sep, void* parts_ptr) {
    auto* parts = static_cast<TythonList*>(parts_ptr);
    auto* bytes_parts = TYTHON_FN(list_empty)();
    for (int64_t i = 0; i < parts->len; i++) {
        auto* item = reinterpret_cast<TythonByteArray*>(static_cast<uintptr_t>(parts->data[i]));
        auto* bytes_item = ba_as_bytes(item);
        TYTHON_FN(list_append)(bytes_parts, static_cast<int64_t>(reinterpret_cast<uintptr_t>(bytes_item)));
    }
    auto* out = TYTHON_FN(bytes_join)(ba_as_bytes(sep), bytes_parts);
    return ba_from_bytes(out);
}

TythonByteArray* TYTHON_FN(bytearray_ljust)(TythonByteArray* ba, int64_t width, TythonBytes* fill) {
    return ba_from_bytes(TYTHON_FN(bytes_ljust)(ba_as_bytes(ba), width, fill));
}

TythonByteArray* TYTHON_FN(bytearray_lower)(TythonByteArray* ba) {
    return ba_from_bytes(TYTHON_FN(bytes_lower)(ba_as_bytes(ba)));
}

TythonByteArray* TYTHON_FN(bytearray_lstrip)(TythonByteArray* ba, TythonBytes* chars) {
    return ba_from_bytes(TYTHON_FN(bytes_lstrip)(ba_as_bytes(ba), chars));
}

TythonBytes* TYTHON_FN(bytearray_maketrans)(TythonByteArray* self, TythonBytes* from, TythonBytes* to) {
    return TYTHON_FN(bytes_maketrans)(ba_as_bytes(self), from, to);
}

void* TYTHON_FN(bytearray_partition)(TythonByteArray* ba, TythonBytes* sep) {
    return tuple_bytes_to_bytearray(TYTHON_FN(bytes_partition)(ba_as_bytes(ba), sep));
}

TythonByteArray* TYTHON_FN(bytearray_removeprefix)(TythonByteArray* ba, TythonBytes* prefix) {
    return ba_from_bytes(TYTHON_FN(bytes_removeprefix)(ba_as_bytes(ba), prefix));
}

TythonByteArray* TYTHON_FN(bytearray_removesuffix)(TythonByteArray* ba, TythonBytes* suffix) {
    return ba_from_bytes(TYTHON_FN(bytes_removesuffix)(ba_as_bytes(ba), suffix));
}

TythonByteArray* TYTHON_FN(bytearray_replace)(TythonByteArray* ba, TythonBytes* old_sub, TythonBytes* new_sub) {
    return ba_from_bytes(TYTHON_FN(bytes_replace)(ba_as_bytes(ba), old_sub, new_sub));
}

int64_t TYTHON_FN(bytearray_rfind)(TythonByteArray* ba, TythonBytes* sub) {
    return TYTHON_FN(bytes_rfind)(ba_as_bytes(ba), sub);
}

int64_t TYTHON_FN(bytearray_rindex)(TythonByteArray* ba, TythonBytes* sub) {
    return TYTHON_FN(bytes_rindex)(ba_as_bytes(ba), sub);
}

TythonByteArray* TYTHON_FN(bytearray_rjust)(TythonByteArray* ba, int64_t width, TythonBytes* fill) {
    return ba_from_bytes(TYTHON_FN(bytes_rjust)(ba_as_bytes(ba), width, fill));
}

void* TYTHON_FN(bytearray_rpartition)(TythonByteArray* ba, TythonBytes* sep) {
    return tuple_bytes_to_bytearray(TYTHON_FN(bytes_rpartition)(ba_as_bytes(ba), sep));
}

void* TYTHON_FN(bytearray_rsplit)(TythonByteArray* ba, TythonBytes* sep) {
    return list_bytes_to_bytearray(TYTHON_FN(bytes_rsplit)(ba_as_bytes(ba), sep));
}

TythonByteArray* TYTHON_FN(bytearray_rstrip)(TythonByteArray* ba, TythonBytes* chars) {
    return ba_from_bytes(TYTHON_FN(bytes_rstrip)(ba_as_bytes(ba), chars));
}

void* TYTHON_FN(bytearray_split)(TythonByteArray* ba, TythonBytes* sep) {
    return list_bytes_to_bytearray(TYTHON_FN(bytes_split)(ba_as_bytes(ba), sep));
}

void* TYTHON_FN(bytearray_splitlines)(TythonByteArray* ba) {
    return list_bytes_to_bytearray(TYTHON_FN(bytes_splitlines)(ba_as_bytes(ba)));
}

int64_t TYTHON_FN(bytearray_startswith)(TythonByteArray* ba, TythonBytes* prefix) {
    return TYTHON_FN(bytes_startswith)(ba_as_bytes(ba), prefix);
}

TythonByteArray* TYTHON_FN(bytearray_strip)(TythonByteArray* ba, TythonBytes* chars) {
    return ba_from_bytes(TYTHON_FN(bytes_strip)(ba_as_bytes(ba), chars));
}

TythonByteArray* TYTHON_FN(bytearray_swapcase)(TythonByteArray* ba) {
    return ba_from_bytes(TYTHON_FN(bytes_swapcase)(ba_as_bytes(ba)));
}

TythonByteArray* TYTHON_FN(bytearray_title)(TythonByteArray* ba) {
    return ba_from_bytes(TYTHON_FN(bytes_title)(ba_as_bytes(ba)));
}

TythonByteArray* TYTHON_FN(bytearray_translate)(TythonByteArray* ba, TythonBytes* table) {
    return ba_from_bytes(TYTHON_FN(bytes_translate)(ba_as_bytes(ba), table));
}

TythonByteArray* TYTHON_FN(bytearray_upper)(TythonByteArray* ba) {
    return ba_from_bytes(TYTHON_FN(bytes_upper)(ba_as_bytes(ba)));
}

TythonByteArray* TYTHON_FN(bytearray_zfill)(TythonByteArray* ba, int64_t width) {
    return ba_from_bytes(TYTHON_FN(bytes_zfill)(ba_as_bytes(ba), width));
}
