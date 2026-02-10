def test_pow_int_basic() -> None:
    x: int = 2 ** 10
    print(x)
    assert x == 1024


def test_pow_int_zero_exp() -> None:
    x: int = 5 ** 0
    print(x)
    assert x == 1


def test_pow_int_one_exp() -> None:
    x: int = 5 ** 1
    assert x == 5


def test_pow_int_base_one() -> None:
    x: int = 1 ** 100
    assert x == 1


def test_pow_int_base_zero() -> None:
    x: int = 0 ** 5
    assert x == 0


def test_pow_int_cubed() -> None:
    x: int = 3 ** 3
    print(x)
    assert x == 27


def test_pow_float() -> None:
    x: float = 2.0 ** 3.0
    print(x)
    assert x == 8.0


def test_pow_float_fractional() -> None:
    x: float = 4.0 ** 0.5
    print(x)
    assert x == 2.0


def run_tests() -> None:
    test_pow_int_basic()
    test_pow_int_zero_exp()
    test_pow_int_one_exp()
    test_pow_int_base_one()
    test_pow_int_base_zero()
    test_pow_int_cubed()
    test_pow_float()
    test_pow_float_fractional()
