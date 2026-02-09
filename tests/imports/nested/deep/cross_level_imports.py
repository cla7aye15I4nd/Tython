from . import mid_level
from .deeper import bottom
from .deeper import complex_imports
from .. import top_level
from ... import module_a
from ... import module_d
from . import sibling_a
from . import sibling_b
from ..top_level import top_func as renamed_top_func


def cross_level_func() -> int:
    bottom_val = bottom.bottom_func()
    mid_val = mid_level.mid_func()
    top_val = top_level.top_func()
    module_a_val = module_a.func_a()
    return bottom_val + mid_val + top_val + module_a_val


def test_complex_imports_module() -> int:
    return complex_imports.test_same_dir()


def test_sibling_interaction() -> int:
    a_val = sibling_a.sibling_a_func()
    b_val = sibling_b.sibling_b_func()
    return a_val + b_val


def test_renamed_import() -> int:
    return renamed_top_func()


def test_deep_chain() -> int:
    return complex_imports.complex_chain()


def compute_cross(x: int) -> int:
    a_result = module_a.compute_a(x)
    d_result = module_d.compute_d(a_result)
    return d_result
