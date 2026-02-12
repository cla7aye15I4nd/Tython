def test_plus_eq() -> None:
    x: int = 5
    x += 3
    print(x)
    print('CHECK test_augmented lhs:', x)
    print('CHECK test_augmented rhs:', 8)
    assert x == 8


def test_minus_eq() -> None:
    x: int = 10
    x -= 3
    print(x)
    print('CHECK test_augmented lhs:', x)
    print('CHECK test_augmented rhs:', 7)
    assert x == 7


def test_mul_eq() -> None:
    x: int = 4
    x *= 3
    print(x)
    print('CHECK test_augmented lhs:', x)
    print('CHECK test_augmented rhs:', 12)
    assert x == 12


def test_floordiv_eq() -> None:
    x: int = 7
    x //= 2
    print(x)
    print('CHECK test_augmented lhs:', x)
    print('CHECK test_augmented rhs:', 3)
    assert x == 3


def test_mod_eq() -> None:
    x: int = 10
    x %= 3
    print(x)
    print('CHECK test_augmented lhs:', x)
    print('CHECK test_augmented rhs:', 1)
    assert x == 1


def test_pow_eq() -> None:
    x: int = 2
    x **= 10
    print(x)
    print('CHECK test_augmented lhs:', x)
    print('CHECK test_augmented rhs:', 1024)
    assert x == 1024


def test_plus_eq_float() -> None:
    x: float = 1.5
    x += 2.5
    print(x)
    print('CHECK test_augmented lhs:', x)
    print('CHECK test_augmented rhs:', 4.0)
    assert x == 4.0


def test_accumulate() -> None:
    x: int = 0
    x += 1
    x += 2
    x += 3
    print('CHECK test_augmented lhs:', x)
    print('CHECK test_augmented rhs:', 6)
    assert x == 6


def run_tests() -> None:
    test_plus_eq()
    test_minus_eq()
    test_mul_eq()
    test_floordiv_eq()
    test_mod_eq()
    test_pow_eq()
    test_plus_eq_float()
    test_accumulate()
