"""Module E - imports from nested packages using complex paths."""

# Import from deeply nested module
from .nested.deep.deeper.bottom import bottom_func, BottomClass

# Import from mid-level
from .nested.deep.mid_level import mid_func, get_const

# Import from top-level
from .nested.top_level import top_func, top_const

def module_e_func():
    return "E_" + bottom_func() + "_" + mid_func() + "_" + top_func()

def module_e_class():
    obj = BottomClass()
    return obj.method()

def module_e_const():
    return get_const() + top_const()
