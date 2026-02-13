#include "tython.h"

extern "C" {
    // Entry point function that generated code must provide
    // Weak linkage ensures link error if not provided
    void __tython_user_main(void) __attribute__((weak));

    // Access to thread-local exception tracking from exception.cpp
    extern thread_local void* __tython_last_exception;
}

int main() {
    try {
        __tython_user_main();
        return 0;
    } catch (...) {
        // Retrieve the last thrown exception via thread-local variable
        if (__tython_last_exception) {
            TythonException* exc = static_cast<TythonException*>(__tython_last_exception);
            __tython_print_unhandled(exc->type_tag, exc->message);
            __tython_last_exception = nullptr;
        } else {
            // Fallback for unexpected exceptions
            __tython_print_unhandled(TYTHON_EXC_RUNTIME_ERROR, nullptr);
        }
        return 1;
    }
}
