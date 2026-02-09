from . import module_c
from . import module_a

def func_d() -> int:
    return 4 + module_c.func_c()

def compute_d(x: int) -> int:
    val = module_a.compute_a(x)
    return module_c.compute_c(val)
