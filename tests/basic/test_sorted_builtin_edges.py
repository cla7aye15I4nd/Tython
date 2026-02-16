def test_sorted_str_list() -> None:
    xs: list[str] = ["b", "a", "aa"]
    ys: list[str] = sorted(xs)
    print("CHECK test_sorted_builtin_edges lhs:", ys)
    print("CHECK test_sorted_builtin_edges rhs:", ["a", "aa", "b"])
    assert ys == ["a", "aa", "b"]


def test_sorted_bytes_list() -> None:
    xs: list[bytes] = [b"b", b"a", b"aa"]
    ys: list[bytes] = sorted(xs)
    print("CHECK test_sorted_builtin_edges lhs:", ys)
    print("CHECK test_sorted_builtin_edges rhs:", [b"a", b"aa", b"b"])
    assert ys == [b"a", b"aa", b"b"]


def test_sorted_bytearray_list() -> None:
    xs: list[bytearray] = [bytearray(b"b"), bytearray(b"a"), bytearray(b"aa")]
    ys: list[bytearray] = sorted(xs)
    print("CHECK test_sorted_builtin_edges lhs:", ys)
    print(
        "CHECK test_sorted_builtin_edges rhs:",
        [bytearray(b"a"), bytearray(b"aa"), bytearray(b"b")],
    )
    assert ys == [bytearray(b"a"), bytearray(b"aa"), bytearray(b"b")]


def run_tests() -> None:
    test_sorted_str_list()
    test_sorted_bytes_list()
    test_sorted_bytearray_list()
