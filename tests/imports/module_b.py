"""Module B - imports sibling module A."""
from . import module_a

def func_b():
    return "B" + module_a.func_a()

def compute_b(x):
    return module_a.compute_a(x) * 2
