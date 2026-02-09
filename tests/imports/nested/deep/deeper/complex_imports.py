from . import bottom
from .. import mid_level
from ... import top_level
from .... import module_a
from .... import module_b
from ....module_c import func_c, compute_c
from ...top_level import top_func
from ..mid_level import mid_func
from .bottom import bottom_func


def test_same_dir() -> int:
    return bottom.bottom_func()


def test_parent_import() -> int:
    return mid_level.mid_func()


def test_grandparent_import() -> int:
    return top_level.top_func()


def test_great_grandparent_import() -> int:
    a_result = module_a.func_a()
    b_result = module_b.func_b()
    return a_result + b_result


def test_specific_imports() -> int:
    c_result = func_c()
    top_result = top_func()
    mid_result = mid_func()
    bottom_result = bottom_func()
    return c_result + top_result + mid_result + bottom_result


def complex_chain() -> int:
    return test_same_dir() + test_parent_import() + test_grandparent_import()


def complex_compute(x: int) -> int:
    return compute_c(x) + bottom_func()
