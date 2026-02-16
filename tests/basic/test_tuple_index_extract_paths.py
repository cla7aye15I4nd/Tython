def test_tuple_index_extract_static_paths() -> None:
    t = (10, 20, 30)

    # Constant index path.
    a: int = t[1]
    print("CHECK test_tuple_index_extract_paths lhs:", a)
    print("CHECK test_tuple_index_extract_paths rhs:", 20)
    assert a == 20

    # Unary negative index path.
    b: int = t[-1]
    print("CHECK test_tuple_index_extract_paths lhs:", b)
    print("CHECK test_tuple_index_extract_paths rhs:", 30)
    assert b == 30

    # Non-literal index path (`Name`) for dynamic tuple indexing.
    i: int = 0
    c: int = t[i]
    print("CHECK test_tuple_index_extract_paths lhs:", c)
    print("CHECK test_tuple_index_extract_paths rhs:", 10)
    assert c == 10


def run_tests() -> None:
    test_tuple_index_extract_static_paths()


if __name__ == "__main__":
    run_tests()
