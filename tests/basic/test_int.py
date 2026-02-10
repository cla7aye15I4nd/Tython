def test_int_zero() -> None:
    print(0)
    assert 0 == 0


def test_int_positive() -> None:
    print(42)
    assert 42 == 42


def test_int_negative() -> None:
    x: int = 0 - 7
    print(x)
    assert x == 0 - 7


def test_int_large() -> None:
    x: int = 1000000
    print(x)
    assert x == 1000000


def test_int_add() -> None:
    result: int = 3 + 5
    print(result)
    assert result == 8


def test_int_sub() -> None:
    result: int = 10 - 3
    print(result)
    assert result == 7


def test_int_mul() -> None:
    result: int = 6 * 7
    print(result)
    assert result == 42


def test_int_mod() -> None:
    result: int = 17 % 5
    print(result)
    assert result == 2


def test_int_mod_even() -> None:
    result: int = 10 % 2
    print(result)
    assert result == 0


def test_int_add_negative() -> None:
    result: int = 10 + (0 - 3)
    print(result)
    assert result == 7


def test_int_sub_to_negative() -> None:
    result: int = 3 - 10
    print(result)
    assert result == 0 - 7


def test_int_mul_by_zero() -> None:
    result: int = 42 * 0
    print(result)
    assert result == 0


def test_int_add_identity() -> None:
    result: int = 99 + 0
    print(result)
    assert result == 99


def test_int_mul_identity() -> None:
    result: int = 99 * 1
    print(result)
    assert result == 99


def test_int_chain_add() -> None:
    result: int = 1 + 2 + 3 + 4 + 5
    print(result)
    assert result == 15


def test_int_chain_mul() -> None:
    result: int = 2 * 3 * 4
    print(result)
    assert result == 24


def test_int_mixed_add_sub() -> None:
    result: int = 100 - 30 + 10 - 5
    print(result)
    assert result == 75


def run_tests() -> None:
    test_int_zero()
    test_int_positive()
    test_int_negative()
    test_int_large()
    test_int_add()
    test_int_sub()
    test_int_mul()
    test_int_mod()
    test_int_mod_even()
    test_int_add_negative()
    test_int_sub_to_negative()
    test_int_mul_by_zero()
    test_int_add_identity()
    test_int_mul_identity()
    test_int_chain_add()
    test_int_chain_mul()
    test_int_mixed_add_sub()
