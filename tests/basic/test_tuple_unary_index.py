def test_tuple_index_with_unary_plus() -> None:
    t = (10, 20, 30)
    v: int = t[+1]
    print("CHECK test_tuple_unary_index lhs:", v)
    print("CHECK test_tuple_unary_index rhs:", 20)
    assert v == 20


def run_tests() -> None:
    test_tuple_index_with_unary_plus()


if __name__ == "__main__":
    run_tests()
