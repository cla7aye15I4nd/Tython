#include "tython.h"
#include "internal/buf.h"

#include <cctype>
#include <cstdio>
#include <cstdint>
#include <cstring>
#include <vector>

using StrBuf = tython::Buf<char>;

static_assert(sizeof(StrBuf) == sizeof(TythonStr),
              "Buf<char> must be layout-compatible with TythonStr");

static auto* b(TythonStr* p) { return reinterpret_cast<StrBuf*>(p); }
static auto* S(StrBuf* p)    { return reinterpret_cast<TythonStr*>(p); }

/* ── core operations (delegated to Buf<char>) ────────────────────── */

TythonStr* TYTHON_FN(str_new)(const char* data, int64_t len) {
    return S(StrBuf::create(data, len));
}

TythonStr* TYTHON_FN(str_concat)(TythonStr* a, TythonStr* other) {
    return S(b(a)->concat(b(other)));
}

TythonStr* TYTHON_FN(str_repeat)(TythonStr* s, int64_t n) {
    return S(b(s)->repeat(n));
}

int64_t TYTHON_FN(str_len)(TythonStr* s)                          { return b(s)->len; }
TythonStr* TYTHON_FN(str_get_char)(TythonStr* s, int64_t index) {
    int64_t i = index;
    if (i < 0) i += b(s)->len;
    if (i < 0 || i >= b(s)->len) {
        TYTHON_FN(raise)(TYTHON_EXC_INDEX_ERROR, TYTHON_FN(str_new)("string index out of range", 25));
        __builtin_unreachable();
    }
    return TYTHON_FN(str_new)(b(s)->data + i, 1);
}
int64_t TYTHON_FN(str_cmp)(TythonStr* a, TythonStr* other)        { return b(a)->cmp(b(other)); }
int64_t TYTHON_FN(str_eq)(TythonStr* a, TythonStr* other)         { return b(a)->eq(b(other)); }
int64_t TYTHON_FN(str_contains)(TythonStr* hay, TythonStr* needle){ return b(hay)->contains_sub(b(needle)); }

void TYTHON_FN(print_str)(TythonStr* s) {
    std::fwrite(b(s)->data, 1, static_cast<size_t>(b(s)->len), stdout);
}

/* ── conversion helpers ──────────────────────────────────────────── */

struct ParsedNumericFormatSpec {
    bool valid = true;
    bool zero_pad = false;
    bool has_width = false;
    int width = 0;
    bool has_precision = false;
    int precision = 0;
    char ty = '\0';
};

static ParsedNumericFormatSpec parse_numeric_format_spec(TythonStr* spec) {
    ParsedNumericFormatSpec out;
    const char* data = b(spec)->data;
    int64_t len = b(spec)->len;
    int64_t i = 0;

    if (i < len && data[i] == '0') {
        out.zero_pad = true;
        i += 1;
    }

    while (i < len && std::isdigit(static_cast<unsigned char>(data[i]))) {
        out.has_width = true;
        out.width = out.width * 10 + (data[i] - '0');
        if (out.width > 1000000) out.width = 1000000;
        i += 1;
    }

    if (i < len && data[i] == '.') {
        out.has_precision = true;
        i += 1;
        bool saw_digit = false;
        while (i < len && std::isdigit(static_cast<unsigned char>(data[i]))) {
            saw_digit = true;
            out.precision = out.precision * 10 + (data[i] - '0');
            if (out.precision > 1000000) out.precision = 1000000;
            i += 1;
        }
        if (!saw_digit) out.valid = false;
    }

    if (i < len) {
        out.ty = data[i];
        i += 1;
    }

    if (i != len) out.valid = false;
    return out;
}

TythonStr* TYTHON_FN(str_from_int)(int64_t val) {
    char buf[32];
    int n = std::snprintf(buf, sizeof(buf), "%lld", (long long)val);
    return S(StrBuf::create(buf, n));
}

TythonStr* TYTHON_FN(str_from_float)(double val) {
    char buf[64];
    std::snprintf(buf, sizeof(buf), "%.12g", val);
    bool has_dot = false;
    for (int i = 0; buf[i]; i++) {
        if (buf[i] == '.' || buf[i] == 'e' || buf[i] == 'E'
            || buf[i] == 'n' || buf[i] == 'i') {
            has_dot = true;
            break;
        }
    }
    if (!has_dot) {
        auto len = std::strlen(buf);
        buf[len] = '.';
        buf[len + 1] = '0';
        buf[len + 2] = '\0';
    }
    return S(StrBuf::create(buf, static_cast<int64_t>(std::strlen(buf))));
}

TythonStr* TYTHON_FN(str_from_bool)(int64_t val) {
    return val ? S(StrBuf::create("True", 4))
               : S(StrBuf::create("False", 5));
}

TythonStr* TYTHON_FN(str_format_int)(int64_t val, TythonStr* spec) {
    auto parsed = parse_numeric_format_spec(spec);
    if (!parsed.valid) return TYTHON_FN(str_from_int)(val);
    if (parsed.ty != '\0' && parsed.ty != 'd') return TYTHON_FN(str_from_int)(val);
    if (parsed.has_precision) return TYTHON_FN(str_from_int)(val);

    int n = 0;
    if (parsed.has_width) {
        if (parsed.zero_pad) {
            n = std::snprintf(nullptr, 0, "%0*lld", parsed.width, (long long)val);
        } else {
            n = std::snprintf(nullptr, 0, "%*lld", parsed.width, (long long)val);
        }
    } else {
        n = std::snprintf(nullptr, 0, "%lld", (long long)val);
    }
    if (n < 0) return TYTHON_FN(str_from_int)(val);

    std::vector<char> buf(static_cast<size_t>(n) + 1);
    if (parsed.has_width) {
        if (parsed.zero_pad) {
            std::snprintf(buf.data(), buf.size(), "%0*lld", parsed.width, (long long)val);
        } else {
            std::snprintf(buf.data(), buf.size(), "%*lld", parsed.width, (long long)val);
        }
    } else {
        std::snprintf(buf.data(), buf.size(), "%lld", (long long)val);
    }
    return S(StrBuf::create(buf.data(), n));
}

TythonStr* TYTHON_FN(str_format_float)(double val, TythonStr* spec) {
    auto parsed = parse_numeric_format_spec(spec);
    if (!parsed.valid) return TYTHON_FN(str_from_float)(val);

    if (parsed.ty == '\0' && !parsed.has_width && !parsed.has_precision && !parsed.zero_pad) {
        return TYTHON_FN(str_from_float)(val);
    }

    char ty = parsed.ty == '\0' ? 'g' : parsed.ty;
    if (ty != 'f' && ty != 'g') return TYTHON_FN(str_from_float)(val);

    int n = 0;
    if (parsed.has_width && parsed.has_precision) {
        if (parsed.zero_pad) {
            n = (ty == 'f')
                    ? std::snprintf(nullptr, 0, "%0*.*f", parsed.width, parsed.precision, val)
                    : std::snprintf(nullptr, 0, "%0*.*g", parsed.width, parsed.precision, val);
        } else {
            n = (ty == 'f')
                    ? std::snprintf(nullptr, 0, "%*.*f", parsed.width, parsed.precision, val)
                    : std::snprintf(nullptr, 0, "%*.*g", parsed.width, parsed.precision, val);
        }
    } else if (parsed.has_width) {
        if (parsed.zero_pad) {
            n = (ty == 'f') ? std::snprintf(nullptr, 0, "%0*f", parsed.width, val)
                            : std::snprintf(nullptr, 0, "%0*g", parsed.width, val);
        } else {
            n = (ty == 'f') ? std::snprintf(nullptr, 0, "%*f", parsed.width, val)
                            : std::snprintf(nullptr, 0, "%*g", parsed.width, val);
        }
    } else if (parsed.has_precision) {
        n = (ty == 'f') ? std::snprintf(nullptr, 0, "%.*f", parsed.precision, val)
                        : std::snprintf(nullptr, 0, "%.*g", parsed.precision, val);
    } else {
        n = (ty == 'f') ? std::snprintf(nullptr, 0, "%f", val)
                        : std::snprintf(nullptr, 0, "%g", val);
    }
    if (n < 0) return TYTHON_FN(str_from_float)(val);

    std::vector<char> buf(static_cast<size_t>(n) + 1);
    if (parsed.has_width && parsed.has_precision) {
        if (parsed.zero_pad) {
            if (ty == 'f') {
                std::snprintf(buf.data(), buf.size(), "%0*.*f", parsed.width, parsed.precision, val);
            } else {
                std::snprintf(buf.data(), buf.size(), "%0*.*g", parsed.width, parsed.precision, val);
            }
        } else {
            if (ty == 'f') {
                std::snprintf(buf.data(), buf.size(), "%*.*f", parsed.width, parsed.precision, val);
            } else {
                std::snprintf(buf.data(), buf.size(), "%*.*g", parsed.width, parsed.precision, val);
            }
        }
    } else if (parsed.has_width) {
        if (parsed.zero_pad) {
            if (ty == 'f') {
                std::snprintf(buf.data(), buf.size(), "%0*f", parsed.width, val);
            } else {
                std::snprintf(buf.data(), buf.size(), "%0*g", parsed.width, val);
            }
        } else {
            if (ty == 'f') {
                std::snprintf(buf.data(), buf.size(), "%*f", parsed.width, val);
            } else {
                std::snprintf(buf.data(), buf.size(), "%*g", parsed.width, val);
            }
        }
    } else if (parsed.has_precision) {
        if (ty == 'f') {
            std::snprintf(buf.data(), buf.size(), "%.*f", parsed.precision, val);
        } else {
            std::snprintf(buf.data(), buf.size(), "%.*g", parsed.precision, val);
        }
    } else {
        if (ty == 'f') {
            std::snprintf(buf.data(), buf.size(), "%f", val);
        } else {
            std::snprintf(buf.data(), buf.size(), "%g", val);
        }
    }
    return S(StrBuf::create(buf.data(), n));
}

/* ── repr(str) ──────────────────────────────────────────────────── */

TythonStr* TYTHON_FN(repr_str)(TythonStr* s) {
    const char* data = b(s)->data;
    int64_t len = b(s)->len;

    /* Pick delimiter: use " if string contains ' but not ", else ' */
    bool has_sq = false, has_dq = false;
    for (int64_t i = 0; i < len; i++) {
        if (data[i] == '\'') has_sq = true;
        if (data[i] == '"')  has_dq = true;
    }
    char quote = (has_sq && !has_dq) ? '"' : '\'';

    /* Compute output length */
    int64_t n = 2; /* opening + closing quote */
    for (int64_t i = 0; i < len; i++) {
        char c = data[i];
        if (c == '\\' || c == quote)    n += 2;
        else if (c == '\t' || c == '\n' || c == '\r') n += 2;
        else if (c >= 32 && c < 127)    n += 1;
        else n += 4; /* \xNN */
    }

    auto* out = reinterpret_cast<TythonStr*>(
        __tython_malloc(static_cast<int64_t>(sizeof(TythonStr)) + n));
    out->len = n;
    char* p = out->data;
    *p++ = quote;
    for (int64_t i = 0; i < len; i++) {
        char c = data[i];
        if (c == '\\')              { *p++ = '\\'; *p++ = '\\'; }
        else if (c == quote)        { *p++ = '\\'; *p++ = quote; }
        else if (c == '\t')         { *p++ = '\\'; *p++ = 't'; }
        else if (c == '\n')         { *p++ = '\\'; *p++ = 'n'; }
        else if (c == '\r')         { *p++ = '\\'; *p++ = 'r'; }
        else if (c >= 32 && c < 127) { *p++ = c; }
        else {
            static const char hex[] = "0123456789abcdef";
            auto uc = static_cast<uint8_t>(c);
            *p++ = '\\'; *p++ = 'x';
            *p++ = hex[uc >> 4]; *p++ = hex[uc & 0xf];
        }
    }
    *p = quote;
    return out;
}

/* ── convenience string methods used by stdlib-like patterns ─────── */

TythonStr* TYTHON_FN(str_read)(TythonStr* s) {
    return s;
}

TythonStr* TYTHON_FN(str_strip)(TythonStr* s) {
    int64_t start = 0;
    int64_t end = b(s)->len;
    while (start < end) {
        char c = b(s)->data[start];
        if (c != ' ' && c != '\t' && c != '\n' && c != '\r' && c != '\f' && c != '\v') break;
        start++;
    }
    while (end > start) {
        char c = b(s)->data[end - 1];
        if (c != ' ' && c != '\t' && c != '\n' && c != '\r' && c != '\f' && c != '\v') break;
        end--;
    }
    return TYTHON_FN(str_new)(b(s)->data + start, end - start);
}

void* TYTHON_FN(str_split)(TythonStr* s, TythonStr* sep) {
    if (b(sep)->len == 0) {
        TYTHON_FN(raise)(TYTHON_EXC_VALUE_ERROR, TYTHON_FN(str_new)("empty separator", 15));
        __builtin_unreachable();
    }
    auto* out = TYTHON_FN(list_empty)();
    int64_t i = 0;
    int64_t last = 0;
    while (i + b(sep)->len <= b(s)->len) {
        if (std::memcmp(b(s)->data + i, b(sep)->data, static_cast<size_t>(b(sep)->len)) == 0) {
            auto* piece = TYTHON_FN(str_new)(b(s)->data + last, i - last);
            TYTHON_FN(list_append)(out, static_cast<int64_t>(reinterpret_cast<uintptr_t>(piece)));
            i += b(sep)->len;
            last = i;
        } else {
            i++;
        }
    }
    auto* tail = TYTHON_FN(str_new)(b(s)->data + last, b(s)->len - last);
    TYTHON_FN(list_append)(out, static_cast<int64_t>(reinterpret_cast<uintptr_t>(tail)));
    return out;
}

TythonStr* TYTHON_FN(str_join)(TythonStr* sep, void* parts_ptr) {
    auto* parts = static_cast<TythonList*>(parts_ptr);
    if (!parts || parts->len == 0) {
        return TYTHON_FN(str_new)("", 0);
    }

    int64_t total = 0;
    for (int64_t i = 0; i < parts->len; i++) {
        auto* part = reinterpret_cast<TythonStr*>(static_cast<uintptr_t>(parts->data[i]));
        total += part->len;
        if (i > 0) total += sep->len;
    }

    auto* out = reinterpret_cast<TythonStr*>(
        __tython_malloc(static_cast<int64_t>(sizeof(TythonStr)) + total));
    out->len = total;
    char* dst = out->data;
    for (int64_t i = 0; i < parts->len; i++) {
        if (i > 0) {
            std::memcpy(dst, sep->data, static_cast<size_t>(sep->len));
            dst += sep->len;
        }
        auto* part = reinterpret_cast<TythonStr*>(static_cast<uintptr_t>(parts->data[i]));
        std::memcpy(dst, part->data, static_cast<size_t>(part->len));
        dst += part->len;
    }
    return out;
}

int64_t TYTHON_FN(str_hash)(TythonStr* s) {
    uint64_t h = 0xcbf29ce484222325ULL;
    for (int64_t i = 0; i < s->len; i++) {
        h ^= static_cast<uint8_t>(s->data[i]);
        h *= 0x100000001b3ULL;
    }
    return static_cast<int64_t>(h);
}

void* TYTHON_FN(set_from_str)(TythonStr* s) {
    bool seen[256] = {false};
    auto* out = TYTHON_FN(list_empty)();
    for (int64_t i = 0; i < b(s)->len; i++) {
        uint8_t ch = static_cast<uint8_t>(b(s)->data[i]);
        if (!seen[ch]) {
            seen[ch] = true;
            char one = static_cast<char>(ch);
            auto* piece = TYTHON_FN(str_new)(&one, 1);
            TYTHON_FN(list_append)(out, static_cast<int64_t>(reinterpret_cast<uintptr_t>(piece)));
        }
    }
    return out;
}
