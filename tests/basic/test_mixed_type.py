def test_int_plus_float() -> None:
    x: float = 1 + 2.5
    print('CHECK test_mixed_type lhs:', x)
    print('CHECK test_mixed_type rhs:', 3.5)
    assert x == 3.5


def test_float_plus_int() -> None:
    x: float = 2.5 + 1
    print('CHECK test_mixed_type lhs:', x)
    print('CHECK test_mixed_type rhs:', 3.5)
    assert x == 3.5


def test_int_mul_float() -> None:
    x: float = 3 * 2.0
    print('CHECK test_mixed_type lhs:', x)
    print('CHECK test_mixed_type rhs:', 6.0)
    assert x == 6.0


def test_int_sub_float() -> None:
    x: float = 10 - 3.5
    print('CHECK test_mixed_type lhs:', x)
    print('CHECK test_mixed_type rhs:', 6.5)
    assert x == 6.5


def test_int_cmp_float_lt() -> None:
    x: bool = 1 < 1.5
    print('CHECK test_mixed_type lhs:', x)
    print('CHECK test_mixed_type rhs:', True)
    assert x == True


def test_int_cmp_float_gt() -> None:
    x: bool = 2 > 1.5
    print('CHECK test_mixed_type lhs:', x)
    print('CHECK test_mixed_type rhs:', True)
    assert x == True


def test_int_cmp_float_eq() -> None:
    x: bool = 2 == 2.0
    print('CHECK test_mixed_type lhs:', x)
    print('CHECK test_mixed_type rhs:', True)
    assert x == True


def run_tests() -> None:
    test_int_plus_float()
    test_float_plus_int()
    test_int_mul_float()
    test_int_sub_float()
    test_int_cmp_float_lt()
    test_int_cmp_float_gt()
    test_int_cmp_float_eq()
