def test_is_int() -> None:
    x: int = 42
    y: int = 42
    assert x is y
    print("is_int ok")

def test_is_not_int() -> None:
    x: int = 1
    y: int = 2
    assert x is not y
    print("is_not_int ok")

def test_is_bool() -> None:
    a: bool = True
    b: bool = True
    assert a is b
    c: bool = False
    assert a is not c
    print("is_bool ok")

def run_tests() -> None:
    test_is_int()
    test_is_not_int()
    test_is_bool()
