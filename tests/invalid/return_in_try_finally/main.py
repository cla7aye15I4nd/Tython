def foo() -> int:
    try:
        return 1
    finally:
        x: int = 0
    return 0

x: int = foo()
