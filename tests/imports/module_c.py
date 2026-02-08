"""Module C - imports multiple siblings."""
from . import module_a
from . import module_b

def func_c():
    return "C" + module_a.func_a() + module_b.func_b()

def compute_c(x):
    return module_a.compute_a(x) + module_b.compute_b(x)
