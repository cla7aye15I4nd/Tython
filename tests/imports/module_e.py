from .nested.deep.deeper.bottom import bottom_func
from .nested.deep.mid_level import mid_func
from .nested.top_level import top_func

def module_e_func() -> int:
    return 5 + bottom_func() + mid_func() + top_func()

def module_e_compute(x: int) -> int:
    return bottom_func() + x
