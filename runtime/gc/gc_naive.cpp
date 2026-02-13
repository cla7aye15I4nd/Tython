#ifdef TYTHON_GC_NAIVE

#include "gc.h"
#include <algorithm>
#include <cstdio>
#include <cstdlib>
#include <mutex>
#include <vector>

namespace {
std::vector<void*>* allocations = nullptr;
std::mutex gc_mutex;
bool cleanup_done = false;

void ensure_initialized() {
    if (!allocations) {
        allocations = new std::vector<void*>();
        allocations->reserve(10000);
    }
}
} // namespace

extern "C" {

void __tython_gc_init(void) {
    std::lock_guard<std::mutex> lock(gc_mutex);
    ensure_initialized();
}

void __tython_gc_cleanup(void) {
    std::lock_guard<std::mutex> lock(gc_mutex);
    if (cleanup_done || !allocations)
        return;

    // Free all tracked allocations
    for (void* ptr : *allocations) {
        std::free(ptr);
    }
    allocations->clear();
    delete allocations;
    allocations = nullptr;
    cleanup_done = true;
}

void* __tython_gc_malloc(int64_t size) {
    void* ptr = std::malloc(static_cast<size_t>(size));
    if (!ptr) {
        std::fprintf(stderr, "MemoryError: GC allocation failed\n");
        std::exit(1);
    }

    std::lock_guard<std::mutex> lock(gc_mutex);
    ensure_initialized();
    allocations->push_back(ptr);
    return ptr;
}

void* __tython_gc_malloc_atomic(int64_t size) {
    // For naive GC, atomic is the same as regular malloc
    // (no optimization for non-pointer data)
    return __tython_gc_malloc(size);
}

void __tython_gc_free(void* ptr) {
    // For resize operations - remove from tracking and free immediately
    if (!ptr)
        return;

    std::lock_guard<std::mutex> lock(gc_mutex);
    if (!allocations)
        return;  // GC not initialized yet, nothing to free

    auto it = std::find(allocations->begin(), allocations->end(), ptr);
    if (it != allocations->end()) {
        allocations->erase(it);
        std::free(ptr);
    }
    // If pointer not found in tracking, it was either already freed or
    // never allocated by us - don't try to free it
}

// Register cleanup with atexit during static initialization
static void register_cleanup() __attribute__((constructor));
static void register_cleanup() { std::atexit(__tython_gc_cleanup); }

} // extern "C"

#endif // TYTHON_GC_NAIVE
