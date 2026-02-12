def test_abs_int_pos() -> None:
    x: int = abs(5)
    print(x)
    assert x == 5


def test_abs_int_neg() -> None:
    x: int = abs(-5)
    print(x)
    assert x == 5


def test_abs_int_zero() -> None:
    x: int = abs(0)
    assert x == 0


def test_abs_float() -> None:
    x: float = abs(-3.5)
    print(x)
    assert x == 3.5


def test_min_int() -> None:
    x: int = min(3, 7)
    print(x)
    assert x == 3


def test_min_int_equal() -> None:
    x: int = min(5, 5)
    assert x == 5


def test_min_float() -> None:
    x: float = min(2.5, 1.5)
    assert x == 1.5


def test_min_int_variadic() -> None:
    x: int = min(9, 3, 7, 4)
    assert x == 3


def test_max_int() -> None:
    x: int = max(3, 7)
    print(x)
    assert x == 7


def test_max_float() -> None:
    x: float = max(2.5, 1.5)
    assert x == 2.5


def test_max_float_variadic() -> None:
    x: float = max(2.5, 1.5, 9.0, 3.0)
    assert x == 9.0


def test_pow_builtin_int() -> None:
    x: int = pow(2, 10)
    print(x)
    assert x == 1024


def test_pow_builtin_float() -> None:
    x: float = pow(2.0, 3.0)
    assert x == 8.0


def test_round_up() -> None:
    x: int = round(3.7)
    print(x)
    assert x == 4


def test_round_down() -> None:
    x: int = round(3.2)
    print(x)
    assert x == 3


def test_print_variadic() -> None:
    print("vals", 1, 2, 3, True)


def test_print_empty() -> None:
    print()


def test_sorted_float_list() -> None:
    src: list[float] = [3.25, -1.5, 0.0, 2.75]
    ordered: list[float] = sorted(src)
    assert ordered[0] == -1.5
    assert ordered[1] == 0.0
    assert ordered[2] == 2.75
    assert ordered[3] == 3.25
    # sorted() should return a new list, leaving source unchanged
    assert src[0] == 3.25
    assert src[1] == -1.5
    assert src[2] == 0.0
    assert src[3] == 2.75


def run_tests() -> None:
    test_abs_int_pos()
    test_abs_int_neg()
    test_abs_int_zero()
    test_abs_float()
    test_min_int()
    test_min_int_equal()
    test_min_float()
    test_min_int_variadic()
    test_max_int()
    test_max_float()
    test_max_float_variadic()
    test_pow_builtin_int()
    test_pow_builtin_float()
    test_round_up()
    test_round_down()
    test_print_variadic()
    test_print_empty()
    test_sorted_float_list()
