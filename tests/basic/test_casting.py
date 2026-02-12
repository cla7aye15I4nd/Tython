def test_float_to_int_trunc() -> None:
    x: int = int(3.7)
    print(x)
    print('CHECK test_casting lhs:', x)
    print('CHECK test_casting rhs:', 3)
    assert x == 3


def test_float_to_int_exact() -> None:
    x: int = int(5.0)
    print(x)
    print('CHECK test_casting lhs:', x)
    print('CHECK test_casting rhs:', 5)
    assert x == 5


def test_float_to_int_zero() -> None:
    x: int = int(0.9)
    print(x)
    print('CHECK test_casting lhs:', x)
    print('CHECK test_casting rhs:', 0)
    assert x == 0


def test_float_to_int_negative() -> None:
    x: int = int(-3.7)
    print(x)
    print('CHECK test_casting lhs:', x)
    print('CHECK test_casting rhs:', -3)
    assert x == -3


def test_int_to_float() -> None:
    x: float = float(42)
    y: int = int(x)
    print(y)
    print('CHECK test_casting lhs:', y)
    print('CHECK test_casting rhs:', 42)
    assert y == 42


def test_int_to_float_zero() -> None:
    x: float = float(0)
    y: int = int(x)
    print('CHECK test_casting lhs:', y)
    print('CHECK test_casting rhs:', 0)
    assert y == 0


def test_float_to_bool_nonzero() -> None:
    x: bool = bool(3.14)
    print('CHECK test_casting lhs:', x)
    print('CHECK test_casting rhs:', True)
    assert x == True


def test_float_to_bool_zero() -> None:
    x: bool = bool(0.0)
    print('CHECK test_casting lhs:', x)
    print('CHECK test_casting rhs:', False)
    assert x == False


def test_float_to_bool_negative() -> None:
    x: bool = bool(-1.0)
    print('CHECK test_casting lhs:', x)
    print('CHECK test_casting rhs:', True)
    assert x == True


def test_bool_to_float_true() -> None:
    x: float = float(True)
    y: int = int(x)
    print(y)
    print('CHECK test_casting lhs:', y)
    print('CHECK test_casting rhs:', 1)
    assert y == 1


def test_bool_to_float_false() -> None:
    x: float = float(False)
    y: int = int(x)
    print(y)
    print('CHECK test_casting lhs:', y)
    print('CHECK test_casting rhs:', 0)
    assert y == 0


def test_bool_to_int_true() -> None:
    x: int = int(True)
    print('CHECK test_casting lhs:', x)
    print('CHECK test_casting rhs:', 1)
    assert x == 1


def test_bool_to_int_false() -> None:
    x: int = int(False)
    print('CHECK test_casting lhs:', x)
    print('CHECK test_casting rhs:', 0)
    assert x == 0


def test_int_to_bool_nonzero() -> None:
    x: bool = bool(42)
    print('CHECK test_casting lhs:', x)
    print('CHECK test_casting rhs:', True)
    assert x == True


def test_int_to_bool_zero() -> None:
    x: bool = bool(0)
    print('CHECK test_casting lhs:', x)
    print('CHECK test_casting rhs:', False)
    assert x == False


def test_int_to_bool_negative() -> None:
    x: bool = bool(-1)
    print('CHECK test_casting lhs:', x)
    print('CHECK test_casting rhs:', True)
    assert x == True


def test_bool_eq_tt() -> None:
    print('CHECK test_casting lhs:', True)
    print('CHECK test_casting rhs:', True)
    assert True == True


def test_bool_eq_ff() -> None:
    print('CHECK test_casting lhs:', False)
    print('CHECK test_casting rhs:', False)
    assert False == False


def test_bool_eq_tf() -> None:
    x: bool = True == False
    print('CHECK test_casting lhs:', x)
    print('CHECK test_casting rhs:', False)
    assert x == False


def test_bool_neq() -> None:
    x: bool = True != False
    print('CHECK test_casting lhs:', x)
    print('CHECK test_casting rhs:', True)
    assert x == True


def test_int_identity_cast() -> None:
    x: int = 42
    y: int = int(x)
    print('CHECK test_casting lhs:', y)
    print('CHECK test_casting rhs:', 42)
    assert y == 42


def test_float_identity_cast() -> None:
    x: float = 3.14
    y: float = float(x)
    print('CHECK test_casting lhs:', y)
    print('CHECK test_casting rhs:', 3.14)
    assert y == 3.14


def test_bool_identity_cast() -> None:
    x: bool = True
    y: bool = bool(x)
    print('CHECK test_casting lhs:', y)
    print('CHECK test_casting rhs:', True)
    assert y == True


def run_tests() -> None:
    test_float_to_int_trunc()
    test_float_to_int_exact()
    test_float_to_int_zero()
    test_float_to_int_negative()
    test_int_to_float()
    test_int_to_float_zero()
    test_float_to_bool_nonzero()
    test_float_to_bool_zero()
    test_float_to_bool_negative()
    test_bool_to_float_true()
    test_bool_to_float_false()
    test_bool_to_int_true()
    test_bool_to_int_false()
    test_int_to_bool_nonzero()
    test_int_to_bool_zero()
    test_int_to_bool_negative()
    test_bool_eq_tt()
    test_bool_eq_ff()
    test_bool_eq_tf()
    test_bool_neq()
    test_int_identity_cast()
    test_float_identity_cast()
    test_bool_identity_cast()
