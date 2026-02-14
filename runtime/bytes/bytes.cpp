#include "tython.h"
#include "internal/buf.h"

#include <cctype>
#include <cstdio>
#include <cstring>
#include <vector>

using BytesBuf = tython::Buf<uint8_t>;

static_assert(sizeof(BytesBuf) == sizeof(TythonBytes),
              "Buf<uint8_t> must be layout-compatible with TythonBytes");

static auto* u(TythonBytes* p) { return reinterpret_cast<BytesBuf*>(p); }
static auto* B(BytesBuf* p) { return reinterpret_cast<TythonBytes*>(p); }

struct BytesTriple {
    TythonBytes* a;
    TythonBytes* b;
    TythonBytes* c;
};

static TythonBytes* bytes_copy(TythonBytes* b) {
    return B(BytesBuf::create(u(b)->data, u(b)->len));
}

static bool is_alpha_ascii(uint8_t c) {
    return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z');
}

static bool is_lower_ascii(uint8_t c) {
    return c >= 'a' && c <= 'z';
}

static bool is_upper_ascii(uint8_t c) {
    return c >= 'A' && c <= 'Z';
}

static bool is_digit_ascii(uint8_t c) {
    return c >= '0' && c <= '9';
}

static uint8_t to_lower_ascii(uint8_t c) {
    return is_upper_ascii(c) ? static_cast<uint8_t>(c - 'A' + 'a') : c;
}

static uint8_t to_upper_ascii(uint8_t c) {
    return is_lower_ascii(c) ? static_cast<uint8_t>(c - 'a' + 'A') : c;
}

static int64_t find_sub(const uint8_t* hay, int64_t hay_len, const uint8_t* needle, int64_t needle_len) {
    if (needle_len == 0) return 0;
    if (needle_len > hay_len) return -1;
    for (int64_t i = 0; i <= hay_len - needle_len; i++) {
        if (std::memcmp(hay + i, needle, static_cast<size_t>(needle_len)) == 0) {
            return i;
        }
    }
    return -1;
}

static int64_t rfind_sub(const uint8_t* hay, int64_t hay_len, const uint8_t* needle, int64_t needle_len) {
    if (needle_len == 0) return hay_len;
    if (needle_len > hay_len) return -1;
    for (int64_t i = hay_len - needle_len; i >= 0; i--) {
        if (std::memcmp(hay + i, needle, static_cast<size_t>(needle_len)) == 0) {
            return i;
        }
    }
    return -1;
}

static void ensure_fillbyte(TythonBytes* fill) {
    if (u(fill)->len != 1) {
        TYTHON_FN(raise)(TYTHON_EXC_VALUE_ERROR, TYTHON_FN(str_new)("fill byte must be a single byte", 31));
        __builtin_unreachable();
    }
}

static void ensure_sep_non_empty(TythonBytes* sep) {
    if (u(sep)->len == 0) {
        TYTHON_FN(raise)(TYTHON_EXC_VALUE_ERROR, TYTHON_FN(str_new)("empty separator", 15));
        __builtin_unreachable();
    }
}

static TythonBytes* bytes_slice(TythonBytes* b, int64_t start, int64_t end) {
    int64_t len = u(b)->len;
    if (start < 0) start = 0;
    if (end < start) end = start;
    if (end > len) end = len;
    return B(BytesBuf::create(u(b)->data + start, end - start));
}

static void* make_partition_tuple(TythonBytes* a, TythonBytes* b, TythonBytes* c) {
    auto* t = static_cast<BytesTriple*>(__tython_malloc(static_cast<int64_t>(sizeof(BytesTriple))));
    t->a = a;
    t->b = b;
    t->c = c;
    return t;
}

/* core operations */

TythonBytes* TYTHON_FN(bytes_new)(const uint8_t* data, int64_t len) {
    return B(BytesBuf::create(data, len));
}

TythonBytes* TYTHON_FN(bytes_concat)(TythonBytes* a, TythonBytes* other) {
    return B(u(a)->concat(u(other)));
}

TythonBytes* TYTHON_FN(bytes_repeat)(TythonBytes* s, int64_t n) {
    return B(u(s)->repeat(n));
}

int64_t TYTHON_FN(bytes_len)(TythonBytes* bb) { return u(bb)->len; }
int64_t TYTHON_FN(bytes_cmp)(TythonBytes* a, TythonBytes* other) { return u(a)->cmp(u(other)); }
int64_t TYTHON_FN(bytes_eq)(TythonBytes* a, TythonBytes* other) { return u(a)->eq(u(other)); }

/* print */

void print_bytes_repr(const uint8_t* data, int64_t len) {
    std::putchar('b');
    std::putchar('\'');
    for (int64_t i = 0; i < len; i++) {
        uint8_t c = data[i];
        if (c == '\\') {
            std::putchar('\\');
            std::putchar('\\');
        } else if (c == '\'') {
            std::putchar('\\');
            std::putchar('\'');
        } else if (c == '\t') {
            std::putchar('\\');
            std::putchar('t');
        } else if (c == '\n') {
            std::putchar('\\');
            std::putchar('n');
        } else if (c == '\r') {
            std::putchar('\\');
            std::putchar('r');
        } else if (c >= 32 && c < 127) {
            std::putchar(c);
        } else {
            std::printf("\\x%02x", c);
        }
    }
    std::putchar('\'');
}

void TYTHON_FN(print_bytes)(TythonBytes* bb) {
    print_bytes_repr(u(bb)->data, u(bb)->len);
}

/* repr conversion */

int64_t bytes_repr_body_len(const uint8_t* data, int64_t len) {
    int64_t n = 0;
    for (int64_t i = 0; i < len; i++) {
        uint8_t c = data[i];
        if (c == '\\' || c == '\'') n += 2;
        else if (c == '\t' || c == '\n' || c == '\r') n += 2;
        else if (c >= 32 && c < 127) n += 1;
        else n += 4;
    }
    return n;
}

char* bytes_repr_body_write(char* out, const uint8_t* data, int64_t len) {
    for (int64_t i = 0; i < len; i++) {
        uint8_t c = data[i];
        if (c == '\\') {
            *out++ = '\\';
            *out++ = '\\';
        } else if (c == '\'') {
            *out++ = '\\';
            *out++ = '\'';
        } else if (c == '\t') {
            *out++ = '\\';
            *out++ = 't';
        } else if (c == '\n') {
            *out++ = '\\';
            *out++ = 'n';
        } else if (c == '\r') {
            *out++ = '\\';
            *out++ = 'r';
        } else if (c >= 32 && c < 127) {
            *out++ = static_cast<char>(c);
        } else {
            static const char hex[] = "0123456789abcdef";
            *out++ = '\\';
            *out++ = 'x';
            *out++ = hex[c >> 4];
            *out++ = hex[c & 0xf];
        }
    }
    return out;
}

TythonStr* TYTHON_FN(str_from_bytes)(TythonBytes* bb) {
    int64_t body_len = bytes_repr_body_len(u(bb)->data, u(bb)->len);
    int64_t total = 3 + body_len;
    auto* s = reinterpret_cast<TythonStr*>(
        __tython_malloc(static_cast<int64_t>(sizeof(TythonStr)) + total));
    s->len = total;
    char* p = s->data;
    *p++ = 'b';
    *p++ = '\'';
    p = bytes_repr_body_write(p, u(bb)->data, u(bb)->len);
    *p = '\'';
    return s;
}

/* constructors */

TythonBytes* TYTHON_FN(bytes_from_int)(int64_t n) {
    if (n < 0) {
        std::fprintf(stderr, "ValueError: negative count\n");
        std::exit(1);
    }
    auto* buf = BytesBuf::create(nullptr, n);
    std::memset(buf->data, 0, static_cast<size_t>(n));
    return B(buf);
}

TythonBytes* TYTHON_FN(bytes_from_str)(TythonStr* s) {
    return B(BytesBuf::create(reinterpret_cast<const uint8_t*>(s->data), s->len));
}

/* bytes methods */

TythonBytes* TYTHON_FN(bytes_capitalize)(TythonBytes* b) {
    auto* out = B(BytesBuf::create(u(b)->data, u(b)->len));
    bool first_alpha_done = false;
    for (int64_t i = 0; i < u(out)->len; i++) {
        uint8_t c = u(out)->data[i];
        if (!first_alpha_done && is_alpha_ascii(c)) {
            u(out)->data[i] = to_upper_ascii(c);
            first_alpha_done = true;
        } else {
            u(out)->data[i] = to_lower_ascii(c);
        }
    }
    return out;
}

TythonBytes* TYTHON_FN(bytes_center)(TythonBytes* b, int64_t width, TythonBytes* fill) {
    ensure_fillbyte(fill);
    int64_t len = u(b)->len;
    if (width <= len) return bytes_copy(b);
    int64_t pad = width - len;
    int64_t left = pad / 2;
    int64_t right = pad - left;
    auto* out = B(BytesBuf::create(nullptr, width));
    std::memset(u(out)->data, u(fill)->data[0], static_cast<size_t>(left));
    std::memcpy(u(out)->data + left, u(b)->data, static_cast<size_t>(len));
    std::memset(u(out)->data + left + len, u(fill)->data[0], static_cast<size_t>(right));
    return out;
}

int64_t TYTHON_FN(bytes_count)(TythonBytes* b, TythonBytes* sub) {
    if (u(sub)->len == 0) return u(b)->len + 1;
    int64_t n = 0;
    int64_t i = 0;
    while (i <= u(b)->len - u(sub)->len) {
        if (std::memcmp(u(b)->data + i, u(sub)->data, static_cast<size_t>(u(sub)->len)) == 0) {
            n++;
            i += u(sub)->len;
        } else {
            i++;
        }
    }
    return n;
}

TythonStr* TYTHON_FN(bytes_decode)(TythonBytes* b) {
    return TYTHON_FN(str_new)(reinterpret_cast<const char*>(u(b)->data), u(b)->len);
}

int64_t TYTHON_FN(bytes_endswith)(TythonBytes* b, TythonBytes* suffix) {
    if (u(suffix)->len > u(b)->len) return 0;
    return std::memcmp(
               u(b)->data + (u(b)->len - u(suffix)->len),
               u(suffix)->data,
               static_cast<size_t>(u(suffix)->len)) == 0
               ? 1
               : 0;
}

TythonBytes* TYTHON_FN(bytes_expandtabs)(TythonBytes* b, int64_t tabsize) {
    if (tabsize < 0) tabsize = 0;
    std::vector<uint8_t> out;
    out.reserve(static_cast<size_t>(u(b)->len));
    int64_t col = 0;
    for (int64_t i = 0; i < u(b)->len; i++) {
        uint8_t c = u(b)->data[i];
        if (c == '\t') {
            int64_t spaces = tabsize == 0 ? 0 : (tabsize - (col % tabsize));
            for (int64_t k = 0; k < spaces; k++) out.push_back(' ');
            col += spaces;
        } else {
            out.push_back(c);
            if (c == '\n' || c == '\r') col = 0;
            else col++;
        }
    }
    return B(BytesBuf::create(out.data(), static_cast<int64_t>(out.size())));
}

int64_t TYTHON_FN(bytes_find)(TythonBytes* b, TythonBytes* sub) {
    return find_sub(u(b)->data, u(b)->len, u(sub)->data, u(sub)->len);
}

static int hex_nibble(char c) {
    if (c >= '0' && c <= '9') return c - '0';
    if (c >= 'a' && c <= 'f') return c - 'a' + 10;
    if (c >= 'A' && c <= 'F') return c - 'A' + 10;
    return -1;
}

TythonBytes* TYTHON_FN(bytes_fromhex)(TythonBytes* _self, TythonStr* hex) {
    (void)_self;
    std::vector<uint8_t> out;
    int hi = -1;
    for (int64_t i = 0; i < hex->len; i++) {
        char c = hex->data[i];
        if (std::isspace(static_cast<unsigned char>(c))) continue;
        int v = hex_nibble(c);
        if (v < 0) {
            TYTHON_FN(raise)(TYTHON_EXC_VALUE_ERROR, TYTHON_FN(str_new)("non-hex digit found", 18));
            __builtin_unreachable();
        }
        if (hi < 0) hi = v;
        else {
            out.push_back(static_cast<uint8_t>((hi << 4) | v));
            hi = -1;
        }
    }
    if (hi >= 0) {
        TYTHON_FN(raise)(TYTHON_EXC_VALUE_ERROR, TYTHON_FN(str_new)("fromhex() odd-length string", 27));
        __builtin_unreachable();
    }
    return B(BytesBuf::create(out.data(), static_cast<int64_t>(out.size())));
}

TythonStr* TYTHON_FN(bytes_hex)(TythonBytes* b) {
    static const char digits[] = "0123456789abcdef";
    int64_t out_len = u(b)->len * 2;
    auto* s = reinterpret_cast<TythonStr*>(
        __tython_malloc(static_cast<int64_t>(sizeof(TythonStr)) + out_len));
    s->len = out_len;
    for (int64_t i = 0; i < u(b)->len; i++) {
        uint8_t c = u(b)->data[i];
        s->data[i * 2] = digits[c >> 4];
        s->data[i * 2 + 1] = digits[c & 0xF];
    }
    return s;
}

int64_t TYTHON_FN(bytes_index)(TythonBytes* b, TythonBytes* sub) {
    int64_t idx = TYTHON_FN(bytes_find)(b, sub);
    if (idx < 0) {
        TYTHON_FN(raise)(TYTHON_EXC_VALUE_ERROR, TYTHON_FN(str_new)("subsection not found", 19));
        __builtin_unreachable();
    }
    return idx;
}

int64_t TYTHON_FN(bytes_isalnum)(TythonBytes* b) {
    if (u(b)->len == 0) return 0;
    for (int64_t i = 0; i < u(b)->len; i++) {
        uint8_t c = u(b)->data[i];
        if (!(is_alpha_ascii(c) || is_digit_ascii(c))) return 0;
    }
    return 1;
}

int64_t TYTHON_FN(bytes_isalpha)(TythonBytes* b) {
    if (u(b)->len == 0) return 0;
    for (int64_t i = 0; i < u(b)->len; i++) if (!is_alpha_ascii(u(b)->data[i])) return 0;
    return 1;
}

int64_t TYTHON_FN(bytes_isascii)(TythonBytes* b) {
    for (int64_t i = 0; i < u(b)->len; i++) if (u(b)->data[i] > 127) return 0;
    return 1;
}

int64_t TYTHON_FN(bytes_isdigit)(TythonBytes* b) {
    if (u(b)->len == 0) return 0;
    for (int64_t i = 0; i < u(b)->len; i++) if (!is_digit_ascii(u(b)->data[i])) return 0;
    return 1;
}

int64_t TYTHON_FN(bytes_islower)(TythonBytes* b) {
    bool has_alpha = false;
    for (int64_t i = 0; i < u(b)->len; i++) {
        uint8_t c = u(b)->data[i];
        if (is_upper_ascii(c)) return 0;
        if (is_lower_ascii(c)) has_alpha = true;
    }
    return has_alpha ? 1 : 0;
}

int64_t TYTHON_FN(bytes_isspace)(TythonBytes* b) {
    if (u(b)->len == 0) return 0;
    for (int64_t i = 0; i < u(b)->len; i++) {
        uint8_t c = u(b)->data[i];
        if (!(c == ' ' || c == '\t' || c == '\n' || c == '\r' || c == '\f' || c == '\v')) return 0;
    }
    return 1;
}

int64_t TYTHON_FN(bytes_istitle)(TythonBytes* b) {
    bool saw_cased = false;
    bool need_upper = true;
    for (int64_t i = 0; i < u(b)->len; i++) {
        uint8_t c = u(b)->data[i];
        if (is_alpha_ascii(c)) {
            if (need_upper) {
                if (!is_upper_ascii(c)) return 0;
                need_upper = false;
                saw_cased = true;
            } else {
                if (!is_lower_ascii(c)) return 0;
            }
        } else {
            need_upper = true;
        }
    }
    return saw_cased ? 1 : 0;
}

int64_t TYTHON_FN(bytes_isupper)(TythonBytes* b) {
    bool has_alpha = false;
    for (int64_t i = 0; i < u(b)->len; i++) {
        uint8_t c = u(b)->data[i];
        if (is_lower_ascii(c)) return 0;
        if (is_upper_ascii(c)) has_alpha = true;
    }
    return has_alpha ? 1 : 0;
}

TythonBytes* TYTHON_FN(bytes_join)(TythonBytes* sep, void* parts_ptr) {
    auto* parts = static_cast<TythonList*>(parts_ptr);
    if (!parts || parts->len == 0) return B(BytesBuf::create(nullptr, 0));

    int64_t total = 0;
    for (int64_t i = 0; i < parts->len; i++) {
        auto* p = reinterpret_cast<TythonBytes*>(static_cast<uintptr_t>(parts->data[i]));
        total += u(p)->len;
        if (i > 0) total += u(sep)->len;
    }

    auto* out = B(BytesBuf::create(nullptr, total));
    uint8_t* dst = u(out)->data;
    for (int64_t i = 0; i < parts->len; i++) {
        if (i > 0 && u(sep)->len > 0) {
            std::memcpy(dst, u(sep)->data, static_cast<size_t>(u(sep)->len));
            dst += u(sep)->len;
        }
        auto* p = reinterpret_cast<TythonBytes*>(static_cast<uintptr_t>(parts->data[i]));
        if (u(p)->len > 0) {
            std::memcpy(dst, u(p)->data, static_cast<size_t>(u(p)->len));
            dst += u(p)->len;
        }
    }
    return out;
}

TythonBytes* TYTHON_FN(bytes_ljust)(TythonBytes* b, int64_t width, TythonBytes* fill) {
    ensure_fillbyte(fill);
    if (width <= u(b)->len) return bytes_copy(b);
    auto* out = B(BytesBuf::create(nullptr, width));
    std::memcpy(u(out)->data, u(b)->data, static_cast<size_t>(u(b)->len));
    std::memset(u(out)->data + u(b)->len, u(fill)->data[0], static_cast<size_t>(width - u(b)->len));
    return out;
}

TythonBytes* TYTHON_FN(bytes_lower)(TythonBytes* b) {
    auto* out = B(BytesBuf::create(u(b)->data, u(b)->len));
    for (int64_t i = 0; i < u(out)->len; i++) u(out)->data[i] = to_lower_ascii(u(out)->data[i]);
    return out;
}

TythonBytes* TYTHON_FN(bytes_lstrip)(TythonBytes* b, TythonBytes* chars) {
    bool allow[256] = {false};
    for (int64_t i = 0; i < u(chars)->len; i++) allow[u(chars)->data[i]] = true;
    int64_t i = 0;
    while (i < u(b)->len && allow[u(b)->data[i]]) i++;
    return bytes_slice(b, i, u(b)->len);
}

TythonBytes* TYTHON_FN(bytes_maketrans)(TythonBytes* _self, TythonBytes* from, TythonBytes* to) {
    (void)_self;
    if (u(from)->len != u(to)->len) {
        TYTHON_FN(raise)(TYTHON_EXC_VALUE_ERROR, TYTHON_FN(str_new)("maketrans arguments must have equal length", 40));
        __builtin_unreachable();
    }
    auto* out = B(BytesBuf::create(nullptr, 256));
    for (int i = 0; i < 256; i++) u(out)->data[i] = static_cast<uint8_t>(i);
    for (int64_t i = 0; i < u(from)->len; i++) {
        u(out)->data[u(from)->data[i]] = u(to)->data[i];
    }
    return out;
}

void* TYTHON_FN(bytes_partition)(TythonBytes* b, TythonBytes* sep) {
    ensure_sep_non_empty(sep);
    int64_t pos = TYTHON_FN(bytes_find)(b, sep);
    if (pos < 0) {
        return make_partition_tuple(bytes_copy(b), B(BytesBuf::create(nullptr, 0)), B(BytesBuf::create(nullptr, 0)));
    }
    TythonBytes* left = bytes_slice(b, 0, pos);
    TythonBytes* mid = bytes_copy(sep);
    TythonBytes* right = bytes_slice(b, pos + u(sep)->len, u(b)->len);
    return make_partition_tuple(left, mid, right);
}

TythonBytes* TYTHON_FN(bytes_removeprefix)(TythonBytes* b, TythonBytes* prefix) {
    if (u(prefix)->len <= u(b)->len &&
        std::memcmp(u(b)->data, u(prefix)->data, static_cast<size_t>(u(prefix)->len)) == 0) {
        return bytes_slice(b, u(prefix)->len, u(b)->len);
    }
    return bytes_copy(b);
}

TythonBytes* TYTHON_FN(bytes_removesuffix)(TythonBytes* b, TythonBytes* suffix) {
    if (u(suffix)->len <= u(b)->len &&
        std::memcmp(u(b)->data + (u(b)->len - u(suffix)->len), u(suffix)->data, static_cast<size_t>(u(suffix)->len)) == 0) {
        return bytes_slice(b, 0, u(b)->len - u(suffix)->len);
    }
    return bytes_copy(b);
}

TythonBytes* TYTHON_FN(bytes_replace)(TythonBytes* b, TythonBytes* old_sub, TythonBytes* new_sub) {
    if (u(old_sub)->len == 0) return bytes_copy(b);
    std::vector<uint8_t> out;
    int64_t i = 0;
    while (i < u(b)->len) {
        if (i <= u(b)->len - u(old_sub)->len &&
            std::memcmp(u(b)->data + i, u(old_sub)->data, static_cast<size_t>(u(old_sub)->len)) == 0) {
            out.insert(out.end(), u(new_sub)->data, u(new_sub)->data + u(new_sub)->len);
            i += u(old_sub)->len;
        } else {
            out.push_back(u(b)->data[i]);
            i++;
        }
    }
    return B(BytesBuf::create(out.data(), static_cast<int64_t>(out.size())));
}

int64_t TYTHON_FN(bytes_rfind)(TythonBytes* b, TythonBytes* sub) {
    return rfind_sub(u(b)->data, u(b)->len, u(sub)->data, u(sub)->len);
}

int64_t TYTHON_FN(bytes_rindex)(TythonBytes* b, TythonBytes* sub) {
    int64_t idx = TYTHON_FN(bytes_rfind)(b, sub);
    if (idx < 0) {
        TYTHON_FN(raise)(TYTHON_EXC_VALUE_ERROR, TYTHON_FN(str_new)("subsection not found", 19));
        __builtin_unreachable();
    }
    return idx;
}

TythonBytes* TYTHON_FN(bytes_rjust)(TythonBytes* b, int64_t width, TythonBytes* fill) {
    ensure_fillbyte(fill);
    if (width <= u(b)->len) return bytes_copy(b);
    int64_t pad = width - u(b)->len;
    auto* out = B(BytesBuf::create(nullptr, width));
    std::memset(u(out)->data, u(fill)->data[0], static_cast<size_t>(pad));
    std::memcpy(u(out)->data + pad, u(b)->data, static_cast<size_t>(u(b)->len));
    return out;
}

void* TYTHON_FN(bytes_rpartition)(TythonBytes* b, TythonBytes* sep) {
    ensure_sep_non_empty(sep);
    int64_t pos = TYTHON_FN(bytes_rfind)(b, sep);
    if (pos < 0) {
        return make_partition_tuple(B(BytesBuf::create(nullptr, 0)), B(BytesBuf::create(nullptr, 0)), bytes_copy(b));
    }
    TythonBytes* left = bytes_slice(b, 0, pos);
    TythonBytes* mid = bytes_copy(sep);
    TythonBytes* right = bytes_slice(b, pos + u(sep)->len, u(b)->len);
    return make_partition_tuple(left, mid, right);
}

void* TYTHON_FN(bytes_split)(TythonBytes* b, TythonBytes* sep) {
    ensure_sep_non_empty(sep);
    auto* out = TYTHON_FN(list_empty)();
    int64_t start = 0;
    int64_t i = 0;
    while (i <= u(b)->len - u(sep)->len) {
        if (std::memcmp(u(b)->data + i, u(sep)->data, static_cast<size_t>(u(sep)->len)) == 0) {
            auto* part = bytes_slice(b, start, i);
            TYTHON_FN(list_append)(out, static_cast<int64_t>(reinterpret_cast<uintptr_t>(part)));
            i += u(sep)->len;
            start = i;
        } else {
            i++;
        }
    }
    auto* tail = bytes_slice(b, start, u(b)->len);
    TYTHON_FN(list_append)(out, static_cast<int64_t>(reinterpret_cast<uintptr_t>(tail)));
    return out;
}

void* TYTHON_FN(bytes_rsplit)(TythonBytes* b, TythonBytes* sep) {
    return TYTHON_FN(bytes_split)(b, sep);
}

TythonBytes* TYTHON_FN(bytes_rstrip)(TythonBytes* b, TythonBytes* chars) {
    bool allow[256] = {false};
    for (int64_t i = 0; i < u(chars)->len; i++) allow[u(chars)->data[i]] = true;
    int64_t end = u(b)->len;
    while (end > 0 && allow[u(b)->data[end - 1]]) end--;
    return bytes_slice(b, 0, end);
}

void* TYTHON_FN(bytes_splitlines)(TythonBytes* b) {
    auto* out = TYTHON_FN(list_empty)();
    int64_t start = 0;
    int64_t i = 0;
    while (i < u(b)->len) {
        uint8_t c = u(b)->data[i];
        if (c == '\n' || c == '\r') {
            auto* part = bytes_slice(b, start, i);
            TYTHON_FN(list_append)(out, static_cast<int64_t>(reinterpret_cast<uintptr_t>(part)));
            if (c == '\r' && i + 1 < u(b)->len && u(b)->data[i + 1] == '\n') i++;
            i++;
            start = i;
        } else {
            i++;
        }
    }
    if (start < u(b)->len) {
        auto* part = bytes_slice(b, start, u(b)->len);
        TYTHON_FN(list_append)(out, static_cast<int64_t>(reinterpret_cast<uintptr_t>(part)));
    }
    return out;
}

int64_t TYTHON_FN(bytes_startswith)(TythonBytes* b, TythonBytes* prefix) {
    if (u(prefix)->len > u(b)->len) return 0;
    return std::memcmp(u(b)->data, u(prefix)->data, static_cast<size_t>(u(prefix)->len)) == 0 ? 1 : 0;
}

TythonBytes* TYTHON_FN(bytes_strip)(TythonBytes* b, TythonBytes* chars) {
    auto* tmp = TYTHON_FN(bytes_lstrip)(b, chars);
    return TYTHON_FN(bytes_rstrip)(tmp, chars);
}

TythonBytes* TYTHON_FN(bytes_swapcase)(TythonBytes* b) {
    auto* out = B(BytesBuf::create(u(b)->data, u(b)->len));
    for (int64_t i = 0; i < u(out)->len; i++) {
        uint8_t c = u(out)->data[i];
        if (is_lower_ascii(c)) u(out)->data[i] = to_upper_ascii(c);
        else if (is_upper_ascii(c)) u(out)->data[i] = to_lower_ascii(c);
    }
    return out;
}

TythonBytes* TYTHON_FN(bytes_title)(TythonBytes* b) {
    auto* out = B(BytesBuf::create(u(b)->data, u(b)->len));
    bool new_word = true;
    for (int64_t i = 0; i < u(out)->len; i++) {
        uint8_t c = u(out)->data[i];
        if (is_alpha_ascii(c)) {
            u(out)->data[i] = new_word ? to_upper_ascii(c) : to_lower_ascii(c);
            new_word = false;
        } else {
            new_word = true;
        }
    }
    return out;
}

TythonBytes* TYTHON_FN(bytes_translate)(TythonBytes* b, TythonBytes* table) {
    if (u(table)->len != 256) {
        TYTHON_FN(raise)(TYTHON_EXC_VALUE_ERROR, TYTHON_FN(str_new)("translation table must be 256 bytes", 35));
        __builtin_unreachable();
    }
    auto* out = B(BytesBuf::create(u(b)->data, u(b)->len));
    for (int64_t i = 0; i < u(out)->len; i++) {
        u(out)->data[i] = u(table)->data[u(out)->data[i]];
    }
    return out;
}

TythonBytes* TYTHON_FN(bytes_upper)(TythonBytes* b) {
    auto* out = B(BytesBuf::create(u(b)->data, u(b)->len));
    for (int64_t i = 0; i < u(out)->len; i++) u(out)->data[i] = to_upper_ascii(u(out)->data[i]);
    return out;
}

TythonBytes* TYTHON_FN(bytes_zfill)(TythonBytes* b, int64_t width) {
    if (width <= u(b)->len) return bytes_copy(b);
    int64_t pad = width - u(b)->len;
    auto* out = B(BytesBuf::create(nullptr, width));
    int64_t dst = 0;
    int64_t src = 0;
    if (u(b)->len > 0 && (u(b)->data[0] == '+' || u(b)->data[0] == '-')) {
        u(out)->data[dst++] = u(b)->data[src++];
    }
    std::memset(u(out)->data + dst, '0', static_cast<size_t>(pad));
    dst += pad;
    std::memcpy(u(out)->data + dst, u(b)->data + src, static_cast<size_t>(u(b)->len - src));
    return out;
}

int64_t TYTHON_FN(bytes_get)(TythonBytes* b, int64_t index) {
    int64_t len = u(b)->len;
    if (index < 0 || index >= len) {
        TYTHON_FN(raise)(TYTHON_EXC_INDEX_ERROR, TYTHON_FN(str_new)("bytes index out of range", 25));
        __builtin_unreachable();
    }
    return static_cast<int64_t>(u(b)->data[index]);
}
