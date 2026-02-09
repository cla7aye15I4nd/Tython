"""
Complex imports module - tests various relative import patterns from deep nesting.
This module is 4 levels deep: imports/nested/deep/deeper/
"""

# Import from same directory (.)
from . import bottom

# Import from parent directory (..)
from .. import mid_level

# Import from grandparent directory (...)
from ... import top_level

# Import from great-grandparent directory (....)
from .... import module_a
from .... import module_b

# Import specific items from multiple levels up
from ....module_c import func_c, compute_c
from ...top_level import top_func
from ..mid_level import mid_func
from .bottom import bottom_func, BottomClass


def test_same_dir():
    """Test import from same directory (.)"""
    return bottom.bottom_func()


def test_parent_import():
    """Test import from parent directory (..)"""
    return mid_level.mid_func()


def test_grandparent_import():
    """Test import from grandparent directory (...)"""
    return top_level.top_func()


def test_great_grandparent_import():
    """Test import from great-grandparent directory (....)"""
    a_result = module_a.func_a()
    b_result = module_b.func_b()
    return f"complex_{a_result}_{b_result}"


def test_specific_imports():
    """Test specific function imports from various levels"""
    c_result = func_c()
    top_result = top_func()
    mid_result = mid_func()
    bottom_result = bottom_func()
    return f"specific_{c_result}_{top_result}_{mid_result}_{bottom_result}"


def test_class_import():
    """Test class import from same directory"""
    obj = BottomClass()
    return obj.method()


def complex_chain():
    """Chain multiple imports together"""
    return f"chain_{test_same_dir()}_{test_parent_import()}_{test_grandparent_import()}"
