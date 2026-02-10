def test_bool_true() -> None:
    x: int = int(True)
    print(x)
    assert x == 1


def test_bool_false() -> None:
    x: int = int(False)
    print(x)
    assert x == 0


def test_bool_from_comparison_true() -> None:
    x: bool = 5 == 5
    result: int = int(x)
    print(result)
    assert result == 1


def test_bool_from_comparison_false() -> None:
    x: bool = 5 == 3
    result: int = int(x)
    print(result)
    assert result == 0


def test_bool_true_in_arithmetic() -> None:
    x: int = int(True) + 10
    print(x)
    assert x == 11


def test_bool_false_in_arithmetic() -> None:
    x: int = int(False) + 10
    print(x)
    assert x == 10


def test_bool_from_gt() -> None:
    x: bool = 10 > 5
    result: int = int(x)
    print(result)
    assert result == 1


def test_bool_from_lt() -> None:
    x: bool = 3 < 7
    result: int = int(x)
    print(result)
    assert result == 1


def test_bool_from_neq_true() -> None:
    x: bool = 3 != 5
    result: int = int(x)
    print(result)
    assert result == 1


def test_bool_from_neq_false() -> None:
    x: bool = 5 != 5
    result: int = int(x)
    print(result)
    assert result == 0


def test_bool_to_int_cast() -> None:
    t: bool = True
    f: bool = False
    result: int = int(t) + int(f)
    print(result)
    assert result == 1


def test_int_to_bool_nonzero() -> None:
    x: bool = bool(42)
    result: int = int(x)
    print(result)
    assert result == 1


def test_int_to_bool_zero() -> None:
    x: bool = bool(0)
    result: int = int(x)
    print(result)
    assert result == 0


def run_tests() -> None:
    test_bool_true()
    test_bool_false()
    test_bool_from_comparison_true()
    test_bool_from_comparison_false()
    test_bool_true_in_arithmetic()
    test_bool_false_in_arithmetic()
    test_bool_from_gt()
    test_bool_from_lt()
    test_bool_from_neq_true()
    test_bool_from_neq_false()
    test_bool_to_int_cast()
    test_int_to_bool_nonzero()
    test_int_to_bool_zero()
