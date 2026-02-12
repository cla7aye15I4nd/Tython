def test_chain_ascending_true() -> None:
    x: bool = 1 < 2 < 3
    print(x)
    print('CHECK test_chained_cmp lhs:', x)
    print('CHECK test_chained_cmp rhs:', True)
    assert x == True


def test_chain_ascending_false() -> None:
    x: bool = 1 < 3 < 2
    print(x)
    print('CHECK test_chained_cmp lhs:', x)
    print('CHECK test_chained_cmp rhs:', False)
    assert x == False


def test_chain_equal() -> None:
    x: bool = 1 <= 1 <= 1
    print(x)
    print('CHECK test_chained_cmp lhs:', x)
    print('CHECK test_chained_cmp rhs:', True)
    assert x == True


def test_chain_descending() -> None:
    x: bool = 3 > 2 > 1
    print(x)
    print('CHECK test_chained_cmp lhs:', x)
    print('CHECK test_chained_cmp rhs:', True)
    assert x == True


def test_chain_with_vars() -> None:
    a: int = 1
    b: int = 5
    c: int = 10
    x: bool = a < b < c
    print('CHECK test_chained_cmp lhs:', x)
    print('CHECK test_chained_cmp rhs:', True)
    assert x == True


def test_chain_three_ops() -> None:
    x: bool = 1 < 2 < 3 < 4
    print('CHECK test_chained_cmp lhs:', x)
    print('CHECK test_chained_cmp rhs:', True)
    assert x == True


def run_tests() -> None:
    test_chain_ascending_true()
    test_chain_ascending_false()
    test_chain_equal()
    test_chain_descending()
    test_chain_with_vars()
    test_chain_three_ops()
