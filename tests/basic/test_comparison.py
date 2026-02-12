def to_int(b: bool) -> int:
    if b:
        return 1
    return 0


def test_eq_true() -> None:
    result: int = to_int(5 == 5)
    print('CHECK test_comparison lhs:', result)
    print('CHECK test_comparison rhs:', 1)
    assert result == 1


def test_eq_false() -> None:
    result: int = to_int(5 == 3)
    print('CHECK test_comparison lhs:', result)
    print('CHECK test_comparison rhs:', 0)
    assert result == 0


def test_neq_true() -> None:
    result: int = to_int(5 != 3)
    print('CHECK test_comparison lhs:', result)
    print('CHECK test_comparison rhs:', 1)
    assert result == 1


def test_neq_false() -> None:
    result: int = to_int(5 != 5)
    print('CHECK test_comparison lhs:', result)
    print('CHECK test_comparison rhs:', 0)
    assert result == 0


def test_lt_true() -> None:
    result: int = to_int(3 < 5)
    print('CHECK test_comparison lhs:', result)
    print('CHECK test_comparison rhs:', 1)
    assert result == 1


def test_lt_false() -> None:
    result: int = to_int(5 < 3)
    print('CHECK test_comparison lhs:', result)
    print('CHECK test_comparison rhs:', 0)
    assert result == 0


def test_lt_equal() -> None:
    result: int = to_int(5 < 5)
    print('CHECK test_comparison lhs:', result)
    print('CHECK test_comparison rhs:', 0)
    assert result == 0


def test_gt_true() -> None:
    result: int = to_int(5 > 3)
    print('CHECK test_comparison lhs:', result)
    print('CHECK test_comparison rhs:', 1)
    assert result == 1


def test_gt_false() -> None:
    result: int = to_int(3 > 5)
    print('CHECK test_comparison lhs:', result)
    print('CHECK test_comparison rhs:', 0)
    assert result == 0


def test_gt_equal() -> None:
    result: int = to_int(5 > 5)
    print('CHECK test_comparison lhs:', result)
    print('CHECK test_comparison rhs:', 0)
    assert result == 0


def test_lte_less() -> None:
    result: int = to_int(3 <= 5)
    print('CHECK test_comparison lhs:', result)
    print('CHECK test_comparison rhs:', 1)
    assert result == 1


def test_lte_equal() -> None:
    result: int = to_int(5 <= 5)
    print('CHECK test_comparison lhs:', result)
    print('CHECK test_comparison rhs:', 1)
    assert result == 1


def test_lte_greater() -> None:
    result: int = to_int(5 <= 3)
    print('CHECK test_comparison lhs:', result)
    print('CHECK test_comparison rhs:', 0)
    assert result == 0


def test_gte_greater() -> None:
    result: int = to_int(5 >= 3)
    print('CHECK test_comparison lhs:', result)
    print('CHECK test_comparison rhs:', 1)
    assert result == 1


def test_gte_equal() -> None:
    result: int = to_int(5 >= 5)
    print('CHECK test_comparison lhs:', result)
    print('CHECK test_comparison rhs:', 1)
    assert result == 1


def test_gte_less() -> None:
    result: int = to_int(3 >= 5)
    print('CHECK test_comparison lhs:', result)
    print('CHECK test_comparison rhs:', 0)
    assert result == 0


def test_cmp_zero() -> None:
    result: int = to_int(0 == 0)
    print('CHECK test_comparison lhs:', result)
    print('CHECK test_comparison rhs:', 1)
    assert result == 1


def test_cmp_negative() -> None:
    neg: int = 0 - 5
    result: int = to_int(neg < 0)
    print('CHECK test_comparison lhs:', result)
    print('CHECK test_comparison rhs:', 1)
    assert result == 1


def test_cmp_negative_ordering() -> None:
    a: int = 0 - 10
    b: int = 0 - 3
    result: int = to_int(a < b)
    print('CHECK test_comparison lhs:', result)
    print('CHECK test_comparison rhs:', 1)
    assert result == 1


def test_cmp_with_arithmetic() -> None:
    result: int = to_int(2 + 3 == 5)
    print('CHECK test_comparison lhs:', result)
    print('CHECK test_comparison rhs:', 1)
    assert result == 1


def test_cmp_variables() -> None:
    x: int = 10
    y: int = 20
    result: int = to_int(x < y)
    print('CHECK test_comparison lhs:', result)
    print('CHECK test_comparison rhs:', 1)
    assert result == 1


def run_tests() -> None:
    test_eq_true()
    test_eq_false()
    test_neq_true()
    test_neq_false()
    test_lt_true()
    test_lt_false()
    test_lt_equal()
    test_gt_true()
    test_gt_false()
    test_gt_equal()
    test_lte_less()
    test_lte_equal()
    test_lte_greater()
    test_gte_greater()
    test_gte_equal()
    test_gte_less()
    test_cmp_zero()
    test_cmp_negative()
    test_cmp_negative_ordering()
    test_cmp_with_arithmetic()
    test_cmp_variables()
