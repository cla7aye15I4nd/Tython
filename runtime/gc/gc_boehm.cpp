#ifdef TYTHON_GC_BOEHM

#include "gc.h"
#include <cstdio>
#include <cstdlib>
#include <gc.h>

extern "C" {

void __tython_gc_init(void) {
    // Initialize Boehm GC
    GC_INIT();
}

void __tython_gc_cleanup(void) {
    // Boehm GC manages its own cleanup
    // Optionally perform a final collection
    GC_gcollect();
}

void* __tython_gc_malloc(int64_t size) {
    // Allocate memory that will be scanned for pointers
    void* ptr = GC_MALLOC(static_cast<size_t>(size));
    if (!ptr) {
        std::fprintf(stderr, "MemoryError: GC allocation failed\n");
        std::exit(1);
    }
    return ptr;
}

void* __tython_gc_malloc_atomic(int64_t size) {
    // Allocate memory for non-pointer data (strings, bytes)
    // This is an optimization - GC won't scan this memory for pointers
    void* ptr = GC_MALLOC_ATOMIC(static_cast<size_t>(size));
    if (!ptr) {
        std::fprintf(stderr, "MemoryError: GC atomic allocation failed\n");
        std::exit(1);
    }
    return ptr;
}

void __tython_gc_free(void* ptr) {
    // Boehm GC doesn't require explicit frees
    // GC_FREE can be called but is typically not needed
    // We make this a no-op for simplicity
    (void)ptr;
}

} // extern "C"

#endif // TYTHON_GC_BOEHM
