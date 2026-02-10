def to_int(b: bool) -> int:
    if b:
        return 1
    return 0


def test_float_eq_true() -> None:
    result: int = to_int(3.14 == 3.14)
    print(result)
    assert result == 1


def test_float_eq_false() -> None:
    result: int = to_int(3.14 == 2.71)
    print(result)
    assert result == 0


def test_float_neq_true() -> None:
    result: int = to_int(1.5 != 2.5)
    assert result == 1


def test_float_neq_false() -> None:
    result: int = to_int(1.5 != 1.5)
    assert result == 0


def test_float_lt_true() -> None:
    result: int = to_int(1.0 < 2.0)
    assert result == 1


def test_float_lt_false() -> None:
    result: int = to_int(2.0 < 1.0)
    assert result == 0


def test_float_lt_equal() -> None:
    result: int = to_int(2.0 < 2.0)
    assert result == 0


def test_float_gt_true() -> None:
    result: int = to_int(5.5 > 3.3)
    assert result == 1


def test_float_gt_false() -> None:
    result: int = to_int(1.0 > 5.0)
    assert result == 0


def test_float_lte_equal() -> None:
    result: int = to_int(2.0 <= 2.0)
    assert result == 1


def test_float_lte_less() -> None:
    result: int = to_int(1.9 <= 2.0)
    assert result == 1


def test_float_lte_greater() -> None:
    result: int = to_int(2.1 <= 2.0)
    assert result == 0


def test_float_gte_greater() -> None:
    result: int = to_int(3.0 >= 2.9)
    assert result == 1


def test_float_gte_equal() -> None:
    result: int = to_int(2.0 >= 2.0)
    assert result == 1


def test_float_gte_less() -> None:
    result: int = to_int(1.0 >= 2.0)
    assert result == 0


def test_float_cmp_with_vars() -> None:
    a: float = 10.5
    b: float = 20.3
    result: int = to_int(a < b)
    print(result)
    assert result == 1


def test_float_cmp_negative() -> None:
    result: int = to_int(-1.5 < 0.0)
    assert result == 1


def run_tests() -> None:
    test_float_eq_true()
    test_float_eq_false()
    test_float_neq_true()
    test_float_neq_false()
    test_float_lt_true()
    test_float_lt_false()
    test_float_lt_equal()
    test_float_gt_true()
    test_float_gt_false()
    test_float_lte_equal()
    test_float_lte_less()
    test_float_lte_greater()
    test_float_gte_greater()
    test_float_gte_equal()
    test_float_gte_less()
    test_float_cmp_with_vars()
    test_float_cmp_negative()
