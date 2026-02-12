#include "tython.h"
#include "internal/buf.h"

#include <cstdio>
#include <cstring>

using BytesBuf = tython::Buf<uint8_t>;

static_assert(sizeof(BytesBuf) == sizeof(TythonBytes),
              "Buf<uint8_t> must be layout-compatible with TythonBytes");

static auto* u(TythonBytes* p) { return reinterpret_cast<BytesBuf*>(p); }
static auto* B(BytesBuf* p)    { return reinterpret_cast<TythonBytes*>(p); }

/* ── core operations (delegated to Buf<uint8_t>) ─────────────────── */

TythonBytes* TYTHON_FN(bytes_new)(const uint8_t* data, int64_t len) {
    return B(BytesBuf::create(data, len));
}

TythonBytes* TYTHON_FN(bytes_concat)(TythonBytes* a, TythonBytes* other) {
    return B(u(a)->concat(u(other)));
}

TythonBytes* TYTHON_FN(bytes_repeat)(TythonBytes* s, int64_t n) {
    return B(u(s)->repeat(n));
}

int64_t TYTHON_FN(bytes_len)(TythonBytes* bb)                      { return u(bb)->len; }
int64_t TYTHON_FN(bytes_cmp)(TythonBytes* a, TythonBytes* other)   { return u(a)->cmp(u(other)); }
int64_t TYTHON_FN(bytes_eq)(TythonBytes* a, TythonBytes* other)    { return u(a)->eq(u(other)); }

/* ── print ───────────────────────────────────────────────────────── */

void print_bytes_repr(const uint8_t* data, int64_t len) {
    std::putchar('b');
    std::putchar('\'');
    for (int64_t i = 0; i < len; i++) {
        uint8_t c = data[i];
        if (c == '\\')       { std::putchar('\\'); std::putchar('\\'); }
        else if (c == '\'')  { std::putchar('\\'); std::putchar('\''); }
        else if (c == '\t')  { std::putchar('\\'); std::putchar('t'); }
        else if (c == '\n')  { std::putchar('\\'); std::putchar('n'); }
        else if (c == '\r')  { std::putchar('\\'); std::putchar('r'); }
        else if (c >= 32 && c < 127) { std::putchar(c); }
        else { std::printf("\\x%02x", c); }
    }
    std::putchar('\'');
}

void TYTHON_FN(print_bytes)(TythonBytes* bb) {
    print_bytes_repr(u(bb)->data, u(bb)->len);
}

/* ── str/repr conversion ─────────────────────────────────────────── */

int64_t bytes_repr_body_len(const uint8_t* data, int64_t len) {
    int64_t n = 0;
    for (int64_t i = 0; i < len; i++) {
        uint8_t c = data[i];
        if (c == '\\' || c == '\'') n += 2;
        else if (c == '\t' || c == '\n' || c == '\r') n += 2;
        else if (c >= 32 && c < 127) n += 1;
        else n += 4; /* \xNN */
    }
    return n;
}

char* bytes_repr_body_write(char* out, const uint8_t* data, int64_t len) {
    for (int64_t i = 0; i < len; i++) {
        uint8_t c = data[i];
        if (c == '\\')       { *out++ = '\\'; *out++ = '\\'; }
        else if (c == '\'')  { *out++ = '\\'; *out++ = '\''; }
        else if (c == '\t')  { *out++ = '\\'; *out++ = 't'; }
        else if (c == '\n')  { *out++ = '\\'; *out++ = 'n'; }
        else if (c == '\r')  { *out++ = '\\'; *out++ = 'r'; }
        else if (c >= 32 && c < 127) { *out++ = static_cast<char>(c); }
        else {
            static const char hex[] = "0123456789abcdef";
            *out++ = '\\'; *out++ = 'x';
            *out++ = hex[c >> 4]; *out++ = hex[c & 0xf];
        }
    }
    return out;
}

TythonStr* TYTHON_FN(str_from_bytes)(TythonBytes* bb) {
    int64_t body_len = bytes_repr_body_len(u(bb)->data, u(bb)->len);
    int64_t total = 3 + body_len; /* b' + body + ' */
    auto* s = reinterpret_cast<TythonStr*>(
        __tython_malloc(static_cast<int64_t>(sizeof(TythonStr)) + total));
    s->len = total;
    char* p = s->data;
    *p++ = 'b'; *p++ = '\'';
    p = bytes_repr_body_write(p, u(bb)->data, u(bb)->len);
    *p = '\'';
    return s;
}

/* ── conversion helpers ──────────────────────────────────────────── */

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
