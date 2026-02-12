def test_pow_int_basic() -> None:
    x: int = 2 ** 10
    print('CHECK test_pow lhs:', x)
    print('CHECK test_pow rhs:', 1024)
    assert x == 1024


def test_pow_int_zero_exp() -> None:
    x: int = 5 ** 0
    print('CHECK test_pow lhs:', x)
    print('CHECK test_pow rhs:', 1)
    assert x == 1


def test_pow_int_one_exp() -> None:
    x: int = 5 ** 1
    print('CHECK test_pow lhs:', x)
    print('CHECK test_pow rhs:', 5)
    assert x == 5


def test_pow_int_base_one() -> None:
    x: int = 1 ** 100
    print('CHECK test_pow lhs:', x)
    print('CHECK test_pow rhs:', 1)
    assert x == 1


def test_pow_int_base_zero() -> None:
    x: int = 0 ** 5
    print('CHECK test_pow lhs:', x)
    print('CHECK test_pow rhs:', 0)
    assert x == 0


def test_pow_int_cubed() -> None:
    x: int = 3 ** 3
    print('CHECK test_pow lhs:', x)
    print('CHECK test_pow rhs:', 27)
    assert x == 27


def test_pow_float() -> None:
    x: float = 2.0 ** 3.0
    print('CHECK test_pow lhs:', x)
    print('CHECK test_pow rhs:', 8.0)
    assert x == 8.0


def test_pow_float_fractional() -> None:
    x: float = 4.0 ** 0.5
    print('CHECK test_pow lhs:', x)
    print('CHECK test_pow rhs:', 2.0)
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
