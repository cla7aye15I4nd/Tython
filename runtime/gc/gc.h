#ifndef TYTHON_GC_H
#define TYTHON_GC_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// GC strategy selection (compile-time)
// Runtime is built with TYTHON_GC_BOEHM.

// Initialize GC system (called before main)
void __tython_gc_init(void);

// Cleanup GC system.
void __tython_gc_cleanup(void);

// Primary allocation function (GC-managed)
void* __tython_gc_malloc(int64_t size);

// Allocation for atomic/non-pointer data (e.g. strings, bytes).
void* __tython_gc_malloc_atomic(int64_t size);

// Explicit free (used for realloc-like operations).
// Boehm implementation is a no-op.
void __tython_gc_free(void* ptr);

#ifdef __cplusplus
}
#endif

#endif /* TYTHON_GC_H */
