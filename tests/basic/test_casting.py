def test_float_to_int_trunc() -> None:
    x: int = int(3.7)
    print(x)
    assert x == 3


def test_float_to_int_exact() -> None:
    x: int = int(5.0)
    print(x)
    assert x == 5


def test_float_to_int_zero() -> None:
    x: int = int(0.9)
    print(x)
    assert x == 0


def test_float_to_int_negative() -> None:
    x: int = int(-3.7)
    print(x)
    assert x == -3


def test_int_to_float() -> None:
    x: float = float(42)
    y: int = int(x)
    print(y)
    assert y == 42


def test_int_to_float_zero() -> None:
    x: float = float(0)
    y: int = int(x)
    assert y == 0


def test_float_to_bool_nonzero() -> None:
    x: bool = bool(3.14)
    assert x == True


def test_float_to_bool_zero() -> None:
    x: bool = bool(0.0)
    assert x == False


def test_float_to_bool_negative() -> None:
    x: bool = bool(-1.0)
    assert x == True


def test_bool_to_float_true() -> None:
    x: float = float(True)
    y: int = int(x)
    print(y)
    assert y == 1


def test_bool_to_float_false() -> None:
    x: float = float(False)
    y: int = int(x)
    print(y)
    assert y == 0


def test_bool_to_int_true() -> None:
    x: int = int(True)
    assert x == 1


def test_bool_to_int_false() -> None:
    x: int = int(False)
    assert x == 0


def test_int_to_bool_nonzero() -> None:
    x: bool = bool(42)
    assert x == True


def test_int_to_bool_zero() -> None:
    x: bool = bool(0)
    assert x == False


def test_int_to_bool_negative() -> None:
    x: bool = bool(-1)
    assert x == True


def test_bool_eq_tt() -> None:
    assert True == True


def test_bool_eq_ff() -> None:
    assert False == False


def test_bool_eq_tf() -> None:
    x: bool = True == False
    assert x == False


def test_bool_neq() -> None:
    x: bool = True != False
    assert x == True


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
