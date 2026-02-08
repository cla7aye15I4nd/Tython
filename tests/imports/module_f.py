"""Module F - imports entire nested modules."""

# Import entire modules
from .nested.deep.deeper import bottom
from .nested.deep import mid_level
from .nested import top_level

def module_f_func():
    bottom_result = bottom.bottom_func()
    mid_result = mid_level.mid_func()
    top_result = top_level.top_func()
    return f"F_{bottom_result}_{mid_result}_{top_result}"

def module_f_access():
    const = bottom.BOTTOM_CONST
    return const + mid_level.get_const()
