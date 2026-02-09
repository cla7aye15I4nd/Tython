from .nested.deep.deeper import bottom
from .nested.deep import mid_level
from .nested import top_level

def module_f_func() -> int:
    bottom_result = bottom.bottom_func()
    mid_result = mid_level.mid_func()
    top_result = top_level.top_func()
    return 6 + bottom_result + mid_result + top_result

def module_f_compute(x: int) -> int:
    return bottom.bottom_compute(x) + mid_level.mid_compute(x)
