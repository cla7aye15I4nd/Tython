from .deep.mid_level import mid_func, mid_compute

def top_func() -> int:
    return 1 + mid_func()

def top_compute(x: int) -> int:
    return mid_compute(x) + 1
