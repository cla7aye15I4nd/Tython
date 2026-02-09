from . import leaf
from .. import bottom
from ... import mid_level
from .... import top_level
from ..... import module_a
from ..... import module_b
from .....module_c import func_c


def test_same_dir() -> int:
    return leaf.leaf_func()


def test_parent_import() -> int:
    return bottom.bottom_func()


def test_grandparent_import() -> int:
    return mid_level.mid_func()


def test_great_grandparent_import() -> int:
    return top_level.top_func()


def test_five_level_import() -> int:
    return module_a.func_a() + module_b.func_b()


def test_all_levels() -> int:
    return test_same_dir() + test_parent_import() + test_grandparent_import() + test_great_grandparent_import() + test_five_level_import()


def deepest_compute(x: int) -> int:
    return func_c() + leaf.leaf_compute(x)
