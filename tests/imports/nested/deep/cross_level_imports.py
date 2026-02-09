"""
Cross-level imports module - imports from various levels.
This module is 3 levels deep: imports/nested/deep/
"""

# Import from sibling
from . import mid_level

# Import from deeper child module
from .deeper import bottom
from .deeper import complex_imports

# Import from parent (...)
from .. import top_level

# Import from grandparent (...) - top level imports
from ... import module_a
from ... import module_d

# Import sibling modules at the same level
from . import sibling_a
from . import sibling_b

# Specific imports across levels
from ...module_e import module_e_func
from ..top_level import top_func as renamed_top_func


def cross_level_func():
    """Test function that uses imports from various levels"""
    bottom_val = bottom.bottom_func()
    mid_val = mid_level.mid_func()
    top_val = top_level.top_func()
    module_a_val = module_a.func_a()
    return f"cross_{bottom_val}_{mid_val}_{top_val}_{module_a_val}"


def test_complex_imports_module():
    """Test using the complex_imports module"""
    return complex_imports.test_same_dir()


def test_sibling_interaction():
    """Test sibling module imports"""
    a_val = sibling_a.sibling_a_func()
    b_val = sibling_b.sibling_b_func()
    return f"siblings_{a_val}_{b_val}"


def test_renamed_import():
    """Test renamed imports"""
    return renamed_top_func()


def test_deep_chain():
    """Test calling into deeper modules"""
    return complex_imports.complex_chain()


def compute_cross(x):
    """Compute using cross-level imports"""
    a_result = module_a.compute_a(x)
    d_result = module_d.compute_d(a_result)
    return d_result
