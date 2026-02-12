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


def test_repr_primitives() -> None:
    assert repr(42) == "42"
    assert repr(2.5) == "2.5"
    assert repr(True) == "True"
    s: str = repr("hello")
    assert len(s) > 0


def test_sum_float_list() -> None:
    xs: list[float] = [1.5, 2.0, -0.5]
    total: float = sum(xs)
    assert total == 3.0


def test_sum_float_list_with_start() -> None:
    xs: list[float] = [1.5, 2.0]
    total: float = sum(xs, 0.5)
    assert total == 4.0


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


def test_sorted_bool_list() -> None:
    src: list[bool] = [True, False, True, False]
    ordered: list[bool] = sorted(src)
    assert ordered[0] == False
    assert ordered[1] == False
    assert ordered[2] == True
    assert ordered[3] == True
    # sorted() should return a new list, leaving source unchanged
    assert src[0] == True
    assert src[1] == False
    assert src[2] == True
    assert src[3] == False


def test_print_list_bytes_and_bytearray() -> None:
    lb: list[bytes] = [b"a", b"bc"]
    lba: list[bytearray] = [bytearray(b"x"), bytearray(b"yz")]
    print(lb)
    print(lba)


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
    test_repr_primitives()
    test_sum_float_list()
    test_sum_float_list_with_start()
    test_print_variadic()
    test_print_empty()
    test_sorted_float_list()
    test_sorted_bool_list()
    test_print_list_bytes_and_bytearray()
