"""Module G - mixed import styles with nested paths."""

# Import from nested with alias
from .nested.deep.sibling_b import sibling_b_func, call_sibling_a

# Import module with alias
from .nested.deep import sibling_a as sib_a

# Regular sibling import
from . import module_e

def module_g_func():
    result1 = sibling_b_func()
    result2 = call_sibling_a()
    result3 = sib_a.sibling_a_func()
    return f"G_{result1}_{result2}_{result3}"

def module_g_use_e():
    return module_e.module_e_func()
