#ifndef TYTHON_GC_H
#define TYTHON_GC_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// GC Strategy selection (compile-time)
// Define either TYTHON_GC_NAIVE or TYTHON_GC_BOEHM

// Initialize GC system (called before main)
void __tython_gc_init(void);

// Cleanup GC system (called after main for naive GC)
void __tython_gc_cleanup(void);

// Primary allocation function (GC-managed)
void* __tython_gc_malloc(int64_t size);

// Non-GC allocation for atomic/non-pointer data
// (Used for strings, bytes - optimization for Boehm GC)
void* __tython_gc_malloc_atomic(int64_t size);

// Explicit free (only for realloc-like operations)
// In Boehm GC: no-op
// In Naive GC: removes from tracking if it was tracked
void __tython_gc_free(void* ptr);

#ifdef __cplusplus
}
#endif

#endif /* TYTHON_GC_H */
