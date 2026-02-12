def test_neg_int() -> None:
    x: int = -5
    print(x)
    print('CHECK test_unary lhs:', x)
    print('CHECK test_unary rhs:', 0 - 5)
    assert x == 0 - 5


def test_neg_int_var() -> None:
    x: int = 42
    y: int = -x
    print(y)
    print('CHECK test_unary lhs:', y)
    print('CHECK test_unary rhs:', 0 - 42)
    assert y == 0 - 42


def test_neg_float() -> None:
    x: float = -3.14
    print(x)
    print('CHECK test_unary assert expr:', 'x < 0.0')
    assert x < 0.0


def test_neg_float_var() -> None:
    x: float = 2.5
    y: float = -x
    print(y)
    print('CHECK test_unary lhs:', y)
    print('CHECK test_unary rhs:', -2.5)
    assert y == -2.5


def test_not_true() -> None:
    x: bool = not True
    print(x)
    print('CHECK test_unary lhs:', x)
    print('CHECK test_unary rhs:', False)
    assert x == False


def test_not_false() -> None:
    x: bool = not False
    print(x)
    print('CHECK test_unary lhs:', x)
    print('CHECK test_unary rhs:', True)
    assert x == True


def test_not_int_truthy() -> None:
    x: bool = not 42
    print('CHECK test_unary lhs:', x)
    print('CHECK test_unary rhs:', False)
    assert x == False


def test_not_int_falsy() -> None:
    x: bool = not 0
    print('CHECK test_unary lhs:', x)
    print('CHECK test_unary rhs:', True)
    assert x == True


def test_bitnot_zero() -> None:
    x: int = ~0
    print(x)
    print('CHECK test_unary lhs:', x)
    print('CHECK test_unary rhs:', -1)
    assert x == -1


def test_bitnot_value() -> None:
    x: int = ~5
    print(x)
    print('CHECK test_unary lhs:', x)
    print('CHECK test_unary rhs:', -6)
    assert x == -6


def run_tests() -> None:
    test_neg_int()
    test_neg_int_var()
    test_neg_float()
    test_neg_float_var()
    test_not_true()
    test_not_false()
    test_not_int_truthy()
    test_not_int_falsy()
    test_bitnot_zero()
    test_bitnot_value()
