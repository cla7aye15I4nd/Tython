def test_float_zero() -> None:
    x: float = 0.0
    result: int = int(x)
    print(result)
    assert result == 0


def test_float_positive() -> None:
    x: float = 3.5
    result: int = int(x)
    print(result)
    assert result == 3


def test_float_add() -> None:
    x: float = 1.5 + 2.5
    result: int = int(x)
    print(result)
    assert result == 4


def test_float_sub() -> None:
    x: float = 10.0 - 3.0
    result: int = int(x)
    print(result)
    assert result == 7


def test_float_mul() -> None:
    x: float = 2.5 * 4.0
    result: int = int(x)
    print(result)
    assert result == 10


def test_float_div() -> None:
    x: float = 10.0 / 3.0
    result: int = int(x)
    print(result)
    assert result == 3


def test_float_div_exact() -> None:
    x: float = 10.0 / 2.0
    result: int = int(x)
    print(result)
    assert result == 5


def test_float_mod() -> None:
    x: float = 10.0 % 3.0
    result: int = int(x)
    print(result)
    assert result == 1


def test_float_chain() -> None:
    x: float = 1.0 + 2.0 + 3.0
    result: int = int(x)
    print(result)
    assert result == 6


def test_float_mul_add() -> None:
    x: float = 2.0 * 3.0 + 4.0
    result: int = int(x)
    print(result)
    assert result == 10


def test_int_to_float() -> None:
    x: float = float(5)
    result: int = int(x + 3.0)
    print(result)
    assert result == 8


def test_float_large() -> None:
    x: float = 1000.0 * 1000.0
    result: int = int(x)
    print(result)
    assert result == 1000000


def run_tests() -> None:
    test_float_zero()
    test_float_positive()
    test_float_add()
    test_float_sub()
    test_float_mul()
    test_float_div()
    test_float_div_exact()
    test_float_mod()
    test_float_chain()
    test_float_mul_add()
    test_int_to_float()
    test_float_large()
