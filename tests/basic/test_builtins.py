def test_abs_int_pos() -> None:
    x: int = abs(5)
    print(x)
    assert x == 5


def test_abs_int_neg() -> None:
    x: int = abs(-5)
    print(x)
    assert x == 5


def test_abs_int_zero() -> None:
    x: int = abs(0)
    assert x == 0


def test_abs_float() -> None:
    x: float = abs(-3.5)
    print(x)
    assert x == 3.5


def test_min_int() -> None:
    x: int = min(3, 7)
    print(x)
    assert x == 3


def test_min_int_equal() -> None:
    x: int = min(5, 5)
    assert x == 5


def test_min_float() -> None:
    x: float = min(2.5, 1.5)
    assert x == 1.5


def test_max_int() -> None:
    x: int = max(3, 7)
    print(x)
    assert x == 7


def test_max_float() -> None:
    x: float = max(2.5, 1.5)
    assert x == 2.5


def test_pow_builtin_int() -> None:
    x: int = pow(2, 10)
    print(x)
    assert x == 1024


def test_pow_builtin_float() -> None:
    x: float = pow(2.0, 3.0)
    assert x == 8.0


def test_round_up() -> None:
    x: int = round(3.7)
    print(x)
    assert x == 4


def test_round_down() -> None:
    x: int = round(3.2)
    print(x)
    assert x == 3


def run_tests() -> None:
    test_abs_int_pos()
    test_abs_int_neg()
    test_abs_int_zero()
    test_abs_float()
    test_min_int()
    test_min_int_equal()
    test_min_float()
    test_max_int()
    test_max_float()
    test_pow_builtin_int()
    test_pow_builtin_float()
    test_round_up()
    test_round_down()
