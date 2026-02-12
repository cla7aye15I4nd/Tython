#include "tython.h"

#include <cstdio>
#include <cstdlib>

/* C++ ABI functions used for LLVM landingpad exception handling.
   These have C linkage in the Itanium C++ ABI. */
extern "C" {
    void* __cxa_allocate_exception(unsigned long thrown_size);
    void  __cxa_throw(void* thrown_exception, void* tinfo, void (*dest)(void*));
    void* __cxa_begin_catch(void* exc_obj);
    void  __cxa_end_catch(void);

    /* typeinfo for void* — mangled C++ symbol from libstdc++/libc++.
       All Tython exceptions use this single typeinfo; dispatch is by type_tag. */
    extern void* _ZTIPv;
}

void TYTHON_FN(raise)(int64_t type_tag, void* message) {
    auto* exc = static_cast<TythonException*>(
        __cxa_allocate_exception(sizeof(TythonException)));
    exc->type_tag = type_tag;
    exc->message  = static_cast<TythonStr*>(message);
    __cxa_throw(exc, &_ZTIPv, nullptr);
    __builtin_unreachable();
}

int64_t TYTHON_FN(caught_type_tag)(void* caught_ptr) {
    return static_cast<TythonException*>(caught_ptr)->type_tag;
}

void* TYTHON_FN(caught_message)(void* caught_ptr) {
    return static_cast<TythonException*>(caught_ptr)->message;
}

int64_t TYTHON_FN(caught_matches)(void* caught_ptr, int64_t type_tag) {
    auto* exc = static_cast<TythonException*>(caught_ptr);

    /* Exception is the base class — matches all non-zero tags */
    if (type_tag == TYTHON_EXC_EXCEPTION)
        return exc->type_tag != TYTHON_EXC_NONE ? 1 : 0;

    if (exc->type_tag == type_tag) return 1;

    /* ArithmeticError catches ZeroDivisionError, OverflowError */
    if (type_tag == TYTHON_EXC_ARITHMETIC_ERROR)
        return (exc->type_tag == TYTHON_EXC_ZERO_DIVISION ||
                exc->type_tag == TYTHON_EXC_OVERFLOW_ERROR) ? 1 : 0;

    /* LookupError catches KeyError, IndexError */
    if (type_tag == TYTHON_EXC_LOOKUP_ERROR)
        return (exc->type_tag == TYTHON_EXC_KEY_ERROR ||
                exc->type_tag == TYTHON_EXC_INDEX_ERROR) ? 1 : 0;

    /* OSError catches FileNotFoundError, PermissionError */
    if (type_tag == TYTHON_EXC_OS_ERROR)
        return (exc->type_tag == TYTHON_EXC_FILE_NOT_FOUND ||
                exc->type_tag == TYTHON_EXC_PERMISSION_ERROR) ? 1 : 0;

    /* ImportError catches ModuleNotFoundError */
    if (type_tag == TYTHON_EXC_IMPORT_ERROR)
        return (exc->type_tag == TYTHON_EXC_MODULE_NOT_FOUND) ? 1 : 0;

    return 0;
}

void TYTHON_FN(print_unhandled)(int64_t type_tag, void* message) {
    const char* name = "Exception";
    switch (type_tag) {
        case TYTHON_EXC_STOP_ITERATION:  name = "StopIteration"; break;
        case TYTHON_EXC_VALUE_ERROR:     name = "ValueError"; break;
        case TYTHON_EXC_TYPE_ERROR:      name = "TypeError"; break;
        case TYTHON_EXC_KEY_ERROR:       name = "KeyError"; break;
        case TYTHON_EXC_RUNTIME_ERROR:   name = "RuntimeError"; break;
        case TYTHON_EXC_ZERO_DIVISION:   name = "ZeroDivisionError"; break;
        case TYTHON_EXC_OVERFLOW_ERROR:  name = "OverflowError"; break;
        case TYTHON_EXC_INDEX_ERROR:     name = "IndexError"; break;
        case TYTHON_EXC_ATTRIBUTE_ERROR: name = "AttributeError"; break;
        case TYTHON_EXC_NOT_IMPLEMENTED: name = "NotImplementedError"; break;
        case TYTHON_EXC_NAME_ERROR:      name = "NameError"; break;
        case TYTHON_EXC_ARITHMETIC_ERROR: name = "ArithmeticError"; break;
        case TYTHON_EXC_LOOKUP_ERROR:     name = "LookupError"; break;
        case TYTHON_EXC_ASSERTION_ERROR:  name = "AssertionError"; break;
        case TYTHON_EXC_IMPORT_ERROR:     name = "ImportError"; break;
        case TYTHON_EXC_MODULE_NOT_FOUND: name = "ModuleNotFoundError"; break;
        case TYTHON_EXC_FILE_NOT_FOUND:   name = "FileNotFoundError"; break;
        case TYTHON_EXC_PERMISSION_ERROR: name = "PermissionError"; break;
        case TYTHON_EXC_OS_ERROR:         name = "OSError"; break;
        default: break;
    }
    if (message) {
        auto* msg = static_cast<TythonStr*>(message);
        std::fprintf(stderr, "%s: %.*s\n", name, static_cast<int>(msg->len), msg->data);
    } else {
        std::fprintf(stderr, "Unhandled %s\n", name);
    }
    std::exit(1);
}
