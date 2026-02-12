def test_abs_int_pos() -> None:
    x: int = abs(5)
    print(x)
    print('CHECK test_builtins lhs:', x)
    print('CHECK test_builtins rhs:', 5)
    assert x == 5


def test_abs_int_neg() -> None:
    x: int = abs(-5)
    print(x)
    print('CHECK test_builtins lhs:', x)
    print('CHECK test_builtins rhs:', 5)
    assert x == 5


def test_abs_int_zero() -> None:
    x: int = abs(0)
    print('CHECK test_builtins lhs:', x)
    print('CHECK test_builtins rhs:', 0)
    assert x == 0


def test_abs_float() -> None:
    x: float = abs(-3.5)
    print(x)
    print('CHECK test_builtins lhs:', x)
    print('CHECK test_builtins rhs:', 3.5)
    assert x == 3.5


def test_min_int() -> None:
    x: int = min(3, 7)
    print(x)
    print('CHECK test_builtins lhs:', x)
    print('CHECK test_builtins rhs:', 3)
    assert x == 3


def test_min_int_equal() -> None:
    x: int = min(5, 5)
    print('CHECK test_builtins lhs:', x)
    print('CHECK test_builtins rhs:', 5)
    assert x == 5


def test_min_float() -> None:
    x: float = min(2.5, 1.5)
    print('CHECK test_builtins lhs:', x)
    print('CHECK test_builtins rhs:', 1.5)
    assert x == 1.5


def test_min_int_variadic() -> None:
    x: int = min(9, 3, 7, 4)
    print('CHECK test_builtins lhs:', x)
    print('CHECK test_builtins rhs:', 3)
    assert x == 3


def test_max_int() -> None:
    x: int = max(3, 7)
    print(x)
    print('CHECK test_builtins lhs:', x)
    print('CHECK test_builtins rhs:', 7)
    assert x == 7


def test_max_float() -> None:
    x: float = max(2.5, 1.5)
    print('CHECK test_builtins lhs:', x)
    print('CHECK test_builtins rhs:', 2.5)
    assert x == 2.5


def test_max_float_variadic() -> None:
    x: float = max(2.5, 1.5, 9.0, 3.0)
    print('CHECK test_builtins lhs:', x)
    print('CHECK test_builtins rhs:', 9.0)
    assert x == 9.0


def test_pow_builtin_int() -> None:
    x: int = pow(2, 10)
    print(x)
    print('CHECK test_builtins lhs:', x)
    print('CHECK test_builtins rhs:', 1024)
    assert x == 1024


def test_pow_builtin_float() -> None:
    x: float = pow(2.0, 3.0)
    print('CHECK test_builtins lhs:', x)
    print('CHECK test_builtins rhs:', 8.0)
    assert x == 8.0


def test_round_up() -> None:
    x: int = round(3.7)
    print(x)
    print('CHECK test_builtins lhs:', x)
    print('CHECK test_builtins rhs:', 4)
    assert x == 4


def test_round_down() -> None:
    x: int = round(3.2)
    print(x)
    print('CHECK test_builtins lhs:', x)
    print('CHECK test_builtins rhs:', 3)
    assert x == 3


def test_repr_primitives() -> None:
    print('CHECK test_builtins lhs expr:', 'repr(42)')
    print('CHECK test_builtins rhs:', '42')
    assert repr(42) == "42"
    print('CHECK test_builtins lhs expr:', 'repr(2.5)')
    print('CHECK test_builtins rhs:', '2.5')
    assert repr(2.5) == "2.5"
    print('CHECK test_builtins lhs expr:', 'repr(True)')
    print('CHECK test_builtins rhs:', 'True')
    assert repr(True) == "True"
    s: str = repr("hello")
    print('CHECK test_builtins assert expr:', 'len(s) > 0')
    assert len(s) > 0


def test_sum_float_list() -> None:
    xs: list[float] = [1.5, 2.0, -0.5]
    total: float = sum(xs)
    print('CHECK test_builtins lhs:', total)
    print('CHECK test_builtins rhs:', 3.0)
    assert total == 3.0


def test_sum_float_list_with_start() -> None:
    xs: list[float] = [1.5, 2.0]
    total: float = sum(xs, 0.5)
    print('CHECK test_builtins lhs:', total)
    print('CHECK test_builtins rhs:', 4.0)
    assert total == 4.0


def test_print_variadic() -> None:
    print("vals", 1, 2, 3, True)


def test_print_empty() -> None:
    print()


def test_sorted_float_list() -> None:
    src: list[float] = [3.25, -1.5, 0.0, 2.75]
    ordered: list[float] = sorted(src)
    print('CHECK test_builtins lhs:', ordered[0])
    print('CHECK test_builtins rhs:', -1.5)
    assert ordered[0] == -1.5
    print('CHECK test_builtins lhs:', ordered[1])
    print('CHECK test_builtins rhs:', 0.0)
    assert ordered[1] == 0.0
    print('CHECK test_builtins lhs:', ordered[2])
    print('CHECK test_builtins rhs:', 2.75)
    assert ordered[2] == 2.75
    print('CHECK test_builtins lhs:', ordered[3])
    print('CHECK test_builtins rhs:', 3.25)
    assert ordered[3] == 3.25
    # sorted() should return a new list, leaving source unchanged
    print('CHECK test_builtins lhs:', src[0])
    print('CHECK test_builtins rhs:', 3.25)
    assert src[0] == 3.25
    print('CHECK test_builtins lhs:', src[1])
    print('CHECK test_builtins rhs:', -1.5)
    assert src[1] == -1.5
    print('CHECK test_builtins lhs:', src[2])
    print('CHECK test_builtins rhs:', 0.0)
    assert src[2] == 0.0
    print('CHECK test_builtins lhs:', src[3])
    print('CHECK test_builtins rhs:', 2.75)
    assert src[3] == 2.75


def test_sorted_bool_list() -> None:
    src: list[bool] = [True, False, True, False]
    ordered: list[bool] = sorted(src)
    print('CHECK test_builtins lhs:', ordered[0])
    print('CHECK test_builtins rhs:', False)
    assert ordered[0] == False
    print('CHECK test_builtins lhs:', ordered[1])
    print('CHECK test_builtins rhs:', False)
    assert ordered[1] == False
    print('CHECK test_builtins lhs:', ordered[2])
    print('CHECK test_builtins rhs:', True)
    assert ordered[2] == True
    print('CHECK test_builtins lhs:', ordered[3])
    print('CHECK test_builtins rhs:', True)
    assert ordered[3] == True
    # sorted() should return a new list, leaving source unchanged
    print('CHECK test_builtins lhs:', src[0])
    print('CHECK test_builtins rhs:', True)
    assert src[0] == True
    print('CHECK test_builtins lhs:', src[1])
    print('CHECK test_builtins rhs:', False)
    assert src[1] == False
    print('CHECK test_builtins lhs:', src[2])
    print('CHECK test_builtins rhs:', True)
    assert src[2] == True
    print('CHECK test_builtins lhs:', src[3])
    print('CHECK test_builtins rhs:', False)
    assert src[3] == False


def test_sorted_str_list() -> None:
    src: list[str] = ["banana", "apple", "cherry"]
    ordered: list[str] = sorted(src)
    print('CHECK test_builtins lhs:', ordered[0])
    print('CHECK test_builtins rhs:', 'apple')
    assert ordered[0] == "apple"
    print('CHECK test_builtins lhs:', ordered[1])
    print('CHECK test_builtins rhs:', 'banana')
    assert ordered[1] == "banana"
    print('CHECK test_builtins lhs:', ordered[2])
    print('CHECK test_builtins rhs:', 'cherry')
    assert ordered[2] == "cherry"
    # sorted() should return a new list, leaving source unchanged
    print('CHECK test_builtins lhs:', src[0])
    print('CHECK test_builtins rhs:', 'banana')
    assert src[0] == "banana"
    print('CHECK test_builtins lhs:', src[1])
    print('CHECK test_builtins rhs:', 'apple')
    assert src[1] == "apple"
    print('CHECK test_builtins lhs:', src[2])
    print('CHECK test_builtins rhs:', 'cherry')
    assert src[2] == "cherry"


def test_list_sort_str() -> None:
    xs: list[str] = ["cherry", "apple", "banana"]
    xs.sort()
    print('CHECK test_builtins lhs:', xs[0])
    print('CHECK test_builtins rhs:', 'apple')
    assert xs[0] == "apple"
    print('CHECK test_builtins lhs:', xs[1])
    print('CHECK test_builtins rhs:', 'banana')
    assert xs[1] == "banana"
    print('CHECK test_builtins lhs:', xs[2])
    print('CHECK test_builtins rhs:', 'cherry')
    assert xs[2] == "cherry"


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
    test_sorted_str_list()
    test_list_sort_str()
    test_print_list_bytes_and_bytearray()
