def test_list_int_literal() -> None:
    xs: list[int] = [1, 2, 3]
    print(xs)
    assert len(xs) == 3


def test_list_float_literal() -> None:
    xs: list[float] = [1.5, 2.5, 3.5]
    print(xs)
    assert len(xs) == 3


def test_list_bool_literal() -> None:
    xs: list[bool] = [True, False, True]
    print(xs)
    assert len(xs) == 3


def test_list_str_literal() -> None:
    xs: list[str] = ["hello", "world"]
    print(xs)
    assert len(xs) == 2


def test_list_empty() -> None:
    xs: list[int] = []
    assert len(xs) == 0


def test_list_get() -> None:
    xs: list[int] = [10, 20, 30]
    assert xs[0] == 10
    assert xs[1] == 20
    assert xs[2] == 30


def test_list_get_negative() -> None:
    xs: list[int] = [10, 20, 30]
    assert xs[-1] == 30
    assert xs[-2] == 20
    assert xs[-3] == 10


def test_list_set() -> None:
    xs: list[int] = [1, 2, 3]
    xs[0] = 10
    xs[2] = 30
    assert xs[0] == 10
    assert xs[1] == 2
    assert xs[2] == 30


def test_list_set_negative() -> None:
    xs: list[int] = [1, 2, 3]
    xs[-1] = 99
    assert xs[2] == 99


def test_list_append_int() -> None:
    xs: list[int] = [1, 2]
    xs.append(3)
    assert len(xs) == 3
    assert xs[2] == 3


def test_list_append_float() -> None:
    xs: list[float] = [1.0, 2.0]
    xs.append(3.0)
    assert len(xs) == 3
    assert xs[2] == 3.0


def test_list_append_bool() -> None:
    xs: list[bool] = [True]
    xs.append(False)
    assert len(xs) == 2
    assert xs[1] == False


def test_list_clear() -> None:
    xs: list[int] = [1, 2, 3]
    xs.clear()
    assert len(xs) == 0


def test_list_truthiness() -> None:
    xs: list[int] = [1, 2, 3]
    if xs:
        print("truthy")
    ys: list[int] = []
    if ys:
        print("should not print")


def test_list_assert() -> None:
    xs: list[int] = [1]
    assert xs


def test_list_augmented_assign() -> None:
    xs: list[int] = [10, 20, 30]
    xs[0] += 5
    xs[1] -= 3
    xs[2] *= 2
    assert xs[0] == 15
    assert xs[1] == 17
    assert xs[2] == 60


def test_list_float_get_set() -> None:
    xs: list[float] = [1.1, 2.2, 3.3]
    assert xs[0] == 1.1
    xs[1] = 9.9
    assert xs[1] == 9.9


def test_list_str_get() -> None:
    xs: list[str] = ["hello", "world"]
    assert xs[0] == "hello"
    assert xs[1] == "world"


def test_list_print_int() -> None:
    print([1, 2, 3])


def test_list_print_float() -> None:
    print([1.5, 2.5])


def test_list_print_bool() -> None:
    print([True, False])


def run_tests() -> None:
    test_list_int_literal()
    test_list_float_literal()
    test_list_bool_literal()
    test_list_str_literal()
    test_list_empty()
    test_list_get()
    test_list_get_negative()
    test_list_set()
    test_list_set_negative()
    test_list_append_int()
    test_list_append_float()
    test_list_append_bool()
    test_list_clear()
    test_list_truthiness()
    test_list_assert()
    test_list_augmented_assign()
    test_list_float_get_set()
    test_list_str_get()
    test_list_print_int()
    test_list_print_float()
    test_list_print_bool()
