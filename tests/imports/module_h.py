from .nested.deep.deeper import bottom
from .nested.deep.deeper import complex_imports
from .nested.deep.deeper.deepest import deepest_imports
from .nested.deep import cross_level_imports
from .nested.deep import mid_level
from .nested import top_level
from . import module_a
from . import module_g


def module_h_deep_access() -> int:
    bottom_val = bottom.bottom_func()
    complex_val = complex_imports.test_same_dir()
    cross_val = cross_level_imports.cross_level_func()
    return 8 + bottom_val + complex_val + cross_val


def module_h_nested_chain() -> int:
    mid_val = mid_level.mid_func()
    top_val = top_level.top_func()
    return 8 + mid_val + top_val


def module_h_complex_chain() -> int:
    same = complex_imports.test_same_dir()
    parent = complex_imports.test_parent_import()
    grandparent = complex_imports.test_grandparent_import()
    great = complex_imports.test_great_grandparent_import()
    return 8 + same + parent + grandparent + great


def module_h_cross_level() -> int:
    cross = cross_level_imports.cross_level_func()
    siblings = cross_level_imports.test_sibling_interaction()
    return 8 + cross + siblings


def module_h_compute(x: int) -> int:
    return cross_level_imports.compute_cross(x)


def module_h_deepest() -> int:
    return deepest_imports.test_all_levels()
