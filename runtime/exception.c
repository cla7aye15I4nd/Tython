#include "tython.h"

/* C++ ABI functions used for LLVM landingpad exception handling */
extern void* __cxa_allocate_exception(unsigned long thrown_size);
extern void  __cxa_throw(void* thrown_exception, void* tinfo, void (*dest)(void*));
extern void* __cxa_begin_catch(void* exc_obj);
extern void  __cxa_end_catch(void);

/* typeinfo for void* from libstdc++/libc++ — used as the exception typeinfo.
   All Tython exceptions use this single typeinfo; dispatch is by type_tag field. */
extern void* _ZTIPv;

void __tython_raise(int64_t type_tag, void* message) {
    TythonException* exc =
        (TythonException*)__cxa_allocate_exception(sizeof(TythonException));
    exc->type_tag = type_tag;
    exc->message  = (TythonStr*)message;
    __cxa_throw(exc, &_ZTIPv, NULL);
    __builtin_unreachable();
}

int64_t __tython_caught_type_tag(void* caught_ptr) {
    TythonException* exc = (TythonException*)caught_ptr;
    return exc->type_tag;
}

void* __tython_caught_message(void* caught_ptr) {
    TythonException* exc = (TythonException*)caught_ptr;
    return (void*)exc->message;
}

int64_t __tython_caught_matches(void* caught_ptr, int64_t type_tag) {
    TythonException* exc = (TythonException*)caught_ptr;
    if (type_tag == TYTHON_EXC_EXCEPTION) {
        /* Exception is the base class — matches all non-zero tags */
        return exc->type_tag != TYTHON_EXC_NONE ? 1 : 0;
    }
    if (exc->type_tag == type_tag) return 1;
    /* Hierarchy: ArithmeticError catches ZeroDivisionError, OverflowError */
    if (type_tag == TYTHON_EXC_ARITHMETIC_ERROR) {
        return (exc->type_tag == TYTHON_EXC_ZERO_DIVISION ||
                exc->type_tag == TYTHON_EXC_OVERFLOW_ERROR) ? 1 : 0;
    }
    /* Hierarchy: LookupError catches KeyError, IndexError */
    if (type_tag == TYTHON_EXC_LOOKUP_ERROR) {
        return (exc->type_tag == TYTHON_EXC_KEY_ERROR ||
                exc->type_tag == TYTHON_EXC_INDEX_ERROR) ? 1 : 0;
    }
    /* Hierarchy: OSError catches FileNotFoundError, PermissionError */
    if (type_tag == TYTHON_EXC_OS_ERROR) {
        return (exc->type_tag == TYTHON_EXC_FILE_NOT_FOUND ||
                exc->type_tag == TYTHON_EXC_PERMISSION_ERROR) ? 1 : 0;
    }
    /* Hierarchy: ImportError catches ModuleNotFoundError */
    if (type_tag == TYTHON_EXC_IMPORT_ERROR) {
        return (exc->type_tag == TYTHON_EXC_MODULE_NOT_FOUND) ? 1 : 0;
    }
    return 0;
}

void __tython_print_unhandled(int64_t type_tag, void* message) {
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
        TythonStr* msg = (TythonStr*)message;
        fprintf(stderr, "%s: %.*s\n", name, (int)msg->len, msg->data);
    } else {
        fprintf(stderr, "Unhandled %s\n", name);
    }
    exit(1);
}
