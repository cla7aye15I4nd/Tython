"""Sibling B in deep package - imports from parent and sibling."""

# Import from parent package
from .. import top_level

# Import from sibling
from . import sibling_a

# Import from deeper child
from .deeper import bottom

def sibling_b_func():
    return "sibling_b_" + top_level.top_func()

def call_sibling_a():
    return sibling_a.sibling_a_func()

def call_bottom():
    return bottom.bottom_func() + "_via_sibling_b"
