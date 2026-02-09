from . import module_a
from . import module_b

def func_c() -> int:
    return 3 + module_a.func_a() + module_b.func_b()

def compute_c(x: int) -> int:
    return module_a.compute_a(x) + module_b.compute_b(x)
