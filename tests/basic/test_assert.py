def test_assert_true() -> None:
    assert True
    print(1)


def test_assert_equality() -> None:
    assert 5 == 5
    print(2)


def test_assert_inequality() -> None:
    assert 3 != 5
    print(3)


def test_assert_greater() -> None:
    assert 10 > 5
    print(4)


def test_assert_less() -> None:
    assert 3 < 8
    print(5)


def test_assert_gte() -> None:
    assert 5 >= 5
    print(6)


def test_assert_lte() -> None:
    assert 5 <= 5
    print(7)


def test_assert_variable() -> None:
    x: int = 42
    assert x == 42
    print(8)


def test_assert_computation() -> None:
    x: int = 3 + 4
    assert x == 7
    print(9)


def test_assert_after_while() -> None:
    total: int = 0
    i: int = 0
    while i < 5:
        total = total + i
        i = i + 1
    assert total == 10
    print(10)


def test_assert_multiple() -> None:
    x: int = 10
    y: int = 20
    assert x == 10
    assert y == 20
    assert x + y == 30
    print(11)


def test_assert_after_if() -> None:
    x: int = 5
    result: int = 0
    if x > 3:
        result = 1
    else:
        result = 2
    assert result == 1
    print(12)


def test_assert_bool_cast() -> None:
    x: bool = bool(1)
    assert x
    print(13)


def run_tests() -> None:
    test_assert_true()
    test_assert_equality()
    test_assert_inequality()
    test_assert_greater()
    test_assert_less()
    test_assert_gte()
    test_assert_lte()
    test_assert_variable()
    test_assert_computation()
    test_assert_after_while()
    test_assert_multiple()
    test_assert_after_if()
    test_assert_bool_cast()
