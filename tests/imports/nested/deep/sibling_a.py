"""Sibling A in deep package."""

from ..top_level import top_func

def sibling_a_func():
    return "sibling_a_calls_" + top_func()
