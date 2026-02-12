#include "tython.h"
#include "internal/buf.h"

#include <cstdio>
#include <cstring>

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
int64_t TYTHON_FN(str_cmp)(TythonStr* a, TythonStr* other)        { return b(a)->cmp(b(other)); }
int64_t TYTHON_FN(str_eq)(TythonStr* a, TythonStr* other)         { return b(a)->eq(b(other)); }
int64_t TYTHON_FN(str_contains)(TythonStr* hay, TythonStr* needle){ return b(hay)->contains_sub(b(needle)); }

void TYTHON_FN(print_str)(TythonStr* s) {
    std::fwrite(b(s)->data, 1, static_cast<size_t>(b(s)->len), stdout);
}

/* ── conversion helpers ──────────────────────────────────────────── */

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
