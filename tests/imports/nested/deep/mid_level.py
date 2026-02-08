"""Mid-level module - imports from deeper level."""

# Relative import from child package
from .deeper.bottom import bottom_func, BottomClass, BOTTOM_CONST

def mid_func():
    return "mid_" + bottom_func()

def use_bottom_class():
    obj = BottomClass()
    return obj.method()

def get_const():
    return BOTTOM_CONST * 2
