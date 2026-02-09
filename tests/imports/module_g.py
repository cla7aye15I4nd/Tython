from .nested.deep.sibling_b import sibling_b_func, call_sibling_a
from .nested.deep import sibling_a as sib_a
from . import module_e

def module_g_func() -> int:
    result1 = sibling_b_func()
    result2 = call_sibling_a()
    result3 = sib_a.sibling_a_func()
    return 7 + result1 + result2 + result3

def module_g_use_e() -> int:
    return module_e.module_e_func()

def module_g_compute(x: int) -> int:
    return sib_a.sibling_a_compute(x) + 7
