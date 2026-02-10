def test_floor_div_positive() -> None:
    x: int = 7 // 2
    print(x)
    assert x == 3


def test_floor_div_negative_dividend() -> None:
    x: int = -7 // 2
    print(x)
    assert x == -4


def test_floor_div_negative_divisor() -> None:
    x: int = 7 // -2
    print(x)
    assert x == -4


def test_floor_div_both_negative() -> None:
    x: int = -7 // -2
    print(x)
    assert x == 3


def test_floor_div_exact() -> None:
    x: int = 10 // 5
    print(x)
    assert x == 2


def test_floor_div_by_one() -> None:
    x: int = 42 // 1
    assert x == 42


def test_floor_div_float() -> None:
    x: float = 7.0 // 2.0
    print(x)
    assert x == 3.0


def test_floor_div_float_negative() -> None:
    x: float = -7.0 // 2.0
    print(x)
    assert x == -4.0


def run_tests() -> None:
    test_floor_div_positive()
    test_floor_div_negative_dividend()
    test_floor_div_negative_divisor()
    test_floor_div_both_negative()
    test_floor_div_exact()
    test_floor_div_by_one()
    test_floor_div_float()
    test_floor_div_float_negative()
