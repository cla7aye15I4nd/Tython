from .. import top_level
from . import sibling_a
from .deeper import bottom

def sibling_b_func() -> int:
    return 600 + top_level.top_func()

def call_sibling_a() -> int:
    return sibling_a.sibling_a_func()

def call_bottom() -> int:
    return bottom.bottom_func() + 600

def sibling_b_compute(x: int) -> int:
    return sibling_a.sibling_a_compute(x) + 100
