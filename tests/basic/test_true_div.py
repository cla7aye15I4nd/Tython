def test_int_true_div() -> None:
    x: float = 7 / 2
    print(x)
    print('CHECK test_true_div lhs:', x)
    print('CHECK test_true_div rhs:', 3.5)
    assert x == 3.5


def test_int_true_div_exact() -> None:
    x: float = 10 / 2
    print(x)
    print('CHECK test_true_div lhs:', x)
    print('CHECK test_true_div rhs:', 5.0)
    assert x == 5.0


def test_int_true_div_negative() -> None:
    x: float = -7 / 2
    print(x)
    print('CHECK test_true_div lhs:', x)
    print('CHECK test_true_div rhs:', -3.5)
    assert x == -3.5


def test_float_div_unchanged() -> None:
    x: float = 7.0 / 2.0
    print(x)
    print('CHECK test_true_div lhs:', x)
    print('CHECK test_true_div rhs:', 3.5)
    assert x == 3.5


def run_tests() -> None:
    test_int_true_div()
    test_int_true_div_exact()
    test_int_true_div_negative()
    test_float_div_unchanged()
