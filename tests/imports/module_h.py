"""
Module H - tests importing from deeply nested modules using absolute-style package paths.
This demonstrates importing from child packages.
"""

# Import from nested child packages
from .nested.deep.deeper import bottom
from .nested.deep.deeper import complex_imports
from .nested.deep import cross_level_imports
from .nested.deep import mid_level
from .nested import top_level

# Also import siblings
from . import module_a
from . import module_g


def module_h_deep_access():
    """Access deeply nested modules from top level"""
    bottom_val = bottom.bottom_func()
    complex_val = complex_imports.test_same_dir()
    cross_val = cross_level_imports.cross_level_func()
    return f"H_{bottom_val}_{complex_val}_{cross_val}"


def module_h_nested_chain():
    """Chain through nested imports"""
    mid_val = mid_level.mid_func()
    top_val = top_level.top_func()
    return f"H_chain_{mid_val}_{top_val}"


def module_h_complex_chain():
    """Use complex_imports module functions"""
    same = complex_imports.test_same_dir()
    parent = complex_imports.test_parent_import()
    grandparent = complex_imports.test_grandparent_import()
    great = complex_imports.test_great_grandparent_import()
    return f"H_{same}_{parent}_{grandparent}_{great}"


def module_h_cross_level():
    """Use cross_level_imports module"""
    cross = cross_level_imports.cross_level_func()
    siblings = cross_level_imports.test_sibling_interaction()
    return f"H_cross_{cross}_{siblings}"


def module_h_compute(x):
    """Compute using nested module"""
    return cross_level_imports.compute_cross(x)
