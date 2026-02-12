def _assert_and_print(label: str, actual: str, expected: str) -> None:
    print(label + " actual  :", actual)
    print(label + " expected:", expected)
    print('CHECK test_str_auto lhs:', actual)
    print('CHECK test_str_auto rhs:', expected)
    assert actual == expected


def test_str_list_int() -> None:
    xs: list[int] = [1, 2, 3]
    s: str = str(xs)
    _assert_and_print("test_str_list_int", s, "[1, 2, 3]")


def test_str_list_float() -> None:
    xs: list[float] = [1.5, 2.5]
    s: str = str(xs)
    _assert_and_print("test_str_list_float", s, "[1.5, 2.5]")


def test_str_list_bool() -> None:
    xs: list[bool] = [True, False]
    s: str = str(xs)
    _assert_and_print("test_str_list_bool", s, "[True, False]")


def test_str_list_str() -> None:
    xs: list[str] = ["hello", "world"]
    s: str = str(xs)
    _assert_and_print("test_str_list_str", s, "['hello', 'world']")


def test_str_list_empty() -> None:
    xs: list[int] = []
    s: str = str(xs)
    _assert_and_print("test_str_list_empty", s, "[]")


def test_str_list_nested() -> None:
    xs: list[list[int]] = [[1, 2], [3, 4]]
    s: str = str(xs)
    _assert_and_print("test_str_list_nested", s, "[[1, 2], [3, 4]]")


def test_str_tuple() -> None:
    t: tuple[int, str] = (1, "hello")
    s: str = str(t)
    _assert_and_print("test_str_tuple", s, "(1, 'hello')")


def test_str_single_tuple() -> None:
    t: tuple[int] = (42,)
    s: str = str(t)
    _assert_and_print("test_str_single_tuple", s, "(42,)")


def test_str_bytes() -> None:
    b: bytes = b"hello"
    s: str = str(b)
    _assert_and_print("test_str_bytes", s, "b'hello'")


def test_str_bytearray() -> None:
    ba: bytearray = bytearray(b"hello")
    s: str = str(ba)
    _assert_and_print("test_str_bytearray", s, "bytearray(b'hello')")


def test_repr_bytes() -> None:
    b: bytes = b"hello"
    s: str = repr(b)
    _assert_and_print("test_repr_bytes", s, "b'hello'")


def test_repr_nested_list() -> None:
    xs: list[list[int]] = [[1, 2], [3, 4]]
    s: str = repr(xs)
    _assert_and_print("test_repr_nested_list", s, "[[1, 2], [3, 4]]")


def test_repr_list_str() -> None:
    xs: list[str] = ["a", "b"]
    s: str = repr(xs)
    _assert_and_print("test_repr_list_str", s, "['a', 'b']")


def test_repr_deeply_nested() -> None:
    t: list[tuple[list[tuple[list[tuple[list[tuple[list[tuple[float, bytes, bytearray, str, tuple[int, str]]]]]]]]]]] = [([([([( [(3.14, b"bytes", bytearray(b"array"), "deep", (1, "hello"))] ,)] ,)] ,)] ,)]
    s: str = repr(t)
    print(repr(t))
    print(str(t))
    print(repr(s))
    print(str(s))
    _assert_and_print(
        "test_repr_deeply_nested",
        s,
        "[([([([([(3.14, b'bytes', bytearray(b'array'), 'deep', (1, 'hello'))],)],)],)],)]",
    )


def test_print_nested_list() -> None:
    xs: list[list[int]] = [[1, 2], [3, 4]]
    print(xs)


def run_tests() -> None:
    test_str_list_int()
    test_str_list_float()
    test_str_list_bool()
    test_str_list_str()
    test_str_list_empty()
    test_str_list_nested()
    test_str_tuple()
    test_str_single_tuple()
    test_str_bytes()
    test_str_bytearray()
    test_repr_bytes()
    test_repr_nested_list()
    test_repr_list_str()
    test_repr_deeply_nested()
    test_print_nested_list()
