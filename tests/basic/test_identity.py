def test_is_int() -> None:
    x: int = 42
    y: int = 42
    print('CHECK test_identity assert expr:', 'x is y')
    assert x is y
    print("is_int ok")

def test_is_not_int() -> None:
    x: int = 1
    y: int = 2
    print('CHECK test_identity assert expr:', 'x is not y')
    assert x is not y
    print("is_not_int ok")

def test_is_bool() -> None:
    a: bool = True
    b: bool = True
    print('CHECK test_identity assert expr:', 'a is b')
    assert a is b
    c: bool = False
    print('CHECK test_identity assert expr:', 'a is not c')
    assert a is not c
    print("is_bool ok")

def run_tests() -> None:
    test_is_int()
    test_is_not_int()
    test_is_bool()
