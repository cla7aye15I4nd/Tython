"""Top-level nested module - imports from sibling and child packages."""

# Relative import from child package
from .deep.mid_level import mid_func, use_bottom_class

# Relative import from deeper child
from .deep.deeper.bottom import BOTTOM_CONST

def top_func():
    return "top_" + mid_func()

def top_use_bottom():
    return use_bottom_class() + "_from_top"

def top_const():
    return BOTTOM_CONST + 100
