from . import module_a

def func_b() -> int:
    return 2 + module_a.func_a()

def compute_b(x: int) -> int:
    return module_a.compute_a(x) * 2
