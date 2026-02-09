from .deeper.bottom import bottom_func, bottom_compute

def mid_func() -> int:
    return 10 + bottom_func()

def mid_compute(x: int) -> int:
    return bottom_compute(x) + 10
