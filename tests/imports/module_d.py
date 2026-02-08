"""Module D - circular-ish dependency (imports C which imports B which imports A)."""
from . import module_c
from . import module_a

def func_d():
    return "D" + module_c.func_c()

def compute_d(x):
    val = module_a.compute_a(x)
    return module_c.compute_c(val)
