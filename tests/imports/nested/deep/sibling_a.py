from ..top_level import top_func

def sibling_a_func() -> int:
    return 500 + top_func()

def sibling_a_compute(x: int) -> int:
    return x + 500
