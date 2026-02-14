def to_int(b: bool) -> int:
    if b:
        return 1
    return 0


def test_eq_true() -> None:
    result: int = to_int(5 == 5)
    print('CHECK test_comparison lhs:', result)
    print('CHECK test_comparison rhs:', 1)
    assert result == 1


def test_eq_false() -> None:
    result: int = to_int(5 == 3)
    print('CHECK test_comparison lhs:', result)
    print('CHECK test_comparison rhs:', 0)
    assert result == 0


def test_neq_true() -> None:
    result: int = to_int(5 != 3)
    print('CHECK test_comparison lhs:', result)
    print('CHECK test_comparison rhs:', 1)
    assert result == 1


def test_neq_false() -> None:
    result: int = to_int(5 != 5)
    print('CHECK test_comparison lhs:', result)
    print('CHECK test_comparison rhs:', 0)
    assert result == 0


def test_lt_true() -> None:
    result: int = to_int(3 < 5)
    print('CHECK test_comparison lhs:', result)
    print('CHECK test_comparison rhs:', 1)
    assert result == 1


def test_lt_false() -> None:
    result: int = to_int(5 < 3)
    print('CHECK test_comparison lhs:', result)
    print('CHECK test_comparison rhs:', 0)
    assert result == 0


def test_lt_equal() -> None:
    result: int = to_int(5 < 5)
    print('CHECK test_comparison lhs:', result)
    print('CHECK test_comparison rhs:', 0)
    assert result == 0


def test_gt_true() -> None:
    result: int = to_int(5 > 3)
    print('CHECK test_comparison lhs:', result)
    print('CHECK test_comparison rhs:', 1)
    assert result == 1


def test_gt_false() -> None:
    result: int = to_int(3 > 5)
    print('CHECK test_comparison lhs:', result)
    print('CHECK test_comparison rhs:', 0)
    assert result == 0


def test_gt_equal() -> None:
    result: int = to_int(5 > 5)
    print('CHECK test_comparison lhs:', result)
    print('CHECK test_comparison rhs:', 0)
    assert result == 0


def test_lte_less() -> None:
    result: int = to_int(3 <= 5)
    print('CHECK test_comparison lhs:', result)
    print('CHECK test_comparison rhs:', 1)
    assert result == 1


def test_lte_equal() -> None:
    result: int = to_int(5 <= 5)
    print('CHECK test_comparison lhs:', result)
    print('CHECK test_comparison rhs:', 1)
    assert result == 1


def test_lte_greater() -> None:
    result: int = to_int(5 <= 3)
    print('CHECK test_comparison lhs:', result)
    print('CHECK test_comparison rhs:', 0)
    assert result == 0


def test_gte_greater() -> None:
    result: int = to_int(5 >= 3)
    print('CHECK test_comparison lhs:', result)
    print('CHECK test_comparison rhs:', 1)
    assert result == 1


def test_gte_equal() -> None:
    result: int = to_int(5 >= 5)
    print('CHECK test_comparison lhs:', result)
    print('CHECK test_comparison rhs:', 1)
    assert result == 1


def test_gte_less() -> None:
    result: int = to_int(3 >= 5)
    print('CHECK test_comparison lhs:', result)
    print('CHECK test_comparison rhs:', 0)
    assert result == 0


def test_cmp_zero() -> None:
    result: int = to_int(0 == 0)
    print('CHECK test_comparison lhs:', result)
    print('CHECK test_comparison rhs:', 1)
    assert result == 1


def test_cmp_negative() -> None:
    neg: int = 0 - 5
    result: int = to_int(neg < 0)
    print('CHECK test_comparison lhs:', result)
    print('CHECK test_comparison rhs:', 1)
    assert result == 1


def test_cmp_negative_ordering() -> None:
    a: int = 0 - 10
    b: int = 0 - 3
    result: int = to_int(a < b)
    print('CHECK test_comparison lhs:', result)
    print('CHECK test_comparison rhs:', 1)
    assert result == 1


def test_cmp_with_arithmetic() -> None:
    result: int = to_int(2 + 3 == 5)
    print('CHECK test_comparison lhs:', result)
    print('CHECK test_comparison rhs:', 1)
    assert result == 1


def test_cmp_variables() -> None:
    x: int = 10
    y: int = 20
    result: int = to_int(x < y)
    print('CHECK test_comparison lhs:', result)
    print('CHECK test_comparison rhs:', 1)
    assert result == 1


def test_list_lexicographic_ordering() -> None:
    a: list[int] = [1, 2]
    b: list[int] = [1, 3]
    c: list[int] = [1, 2]

    print('CHECK test_comparison lhs:', a < b)
    print('CHECK test_comparison rhs:', True)
    assert (a < b) == True
    print('CHECK test_comparison lhs:', b > a)
    print('CHECK test_comparison rhs:', True)
    assert (b > a) == True
    print('CHECK test_comparison lhs:', a <= c)
    print('CHECK test_comparison rhs:', True)
    assert (a <= c) == True
    print('CHECK test_comparison lhs:', b >= c)
    print('CHECK test_comparison rhs:', True)
    assert (b >= c) == True


class Rank:
    n: int

    def __init__(self, n: int) -> None:
        self.n = n

    def __eq__(self, other: "Rank") -> bool:
        return self.n == other.n

    def __lt__(self, other: "Rank") -> bool:
        return self.n < other.n

    def __le__(self, other: "Rank") -> bool:
        return self.n <= other.n

    def __gt__(self, other: "Rank") -> bool:
        return self.n > other.n

    def __ge__(self, other: "Rank") -> bool:
        return self.n >= other.n


class Token:
    n: int

    def __init__(self, n: int) -> None:
        self.n = n

    def __eq__(self, other: "Token") -> bool:
        return self.n == other.n


def test_class_comparison_ordering_and_eq() -> None:
    a: Rank = Rank(1)
    b: Rank = Rank(2)
    c: Rank = Rank(1)

    print('CHECK test_comparison lhs:', a == c)
    print('CHECK test_comparison rhs:', True)
    assert (a == c) == True
    print('CHECK test_comparison lhs:', a != b)
    print('CHECK test_comparison rhs:', True)
    assert (a != b) == True
    print('CHECK test_comparison lhs:', a < b)
    print('CHECK test_comparison rhs:', True)
    assert (a < b) == True
    print('CHECK test_comparison lhs:', a <= b)
    print('CHECK test_comparison rhs:', True)
    assert (a <= b) == True
    print('CHECK test_comparison lhs:', b > a)
    print('CHECK test_comparison rhs:', True)
    assert (b > a) == True
    print('CHECK test_comparison lhs:', b >= a)
    print('CHECK test_comparison rhs:', True)
    assert (b >= a) == True


def test_tuple_eq_matrix() -> None:
    empty_a: tuple[()] = ()
    empty_b: tuple[()] = ()
    print('CHECK test_comparison lhs:', empty_a == empty_b)
    print('CHECK test_comparison rhs:', True)
    assert (empty_a == empty_b) == True

    a: tuple[float, bool, str, bytes, bytearray] = (
        1.5,
        True,
        "ab",
        b"x",
        bytearray(b"y"),
    )
    b: tuple[float, bool, str, bytes, bytearray] = (
        1.5,
        True,
        "ab",
        b"x",
        bytearray(b"y"),
    )
    c: tuple[float, bool, str, bytes, bytearray] = (
        1.5,
        False,
        "ab",
        b"x",
        bytearray(b"y"),
    )
    print('CHECK test_comparison lhs:', a == b)
    print('CHECK test_comparison rhs:', True)
    assert (a == b) == True
    print('CHECK test_comparison lhs:', a != c)
    print('CHECK test_comparison rhs:', True)
    assert (a != c) == True

    t1: tuple[Token] = (Token(7),)
    t2: tuple[Token] = (Token(7),)
    t3: tuple[Token] = (Token(8),)
    print('CHECK test_comparison lhs:', t1 == t2)
    print('CHECK test_comparison rhs:', True)
    assert (t1 == t2) == True
    print('CHECK test_comparison lhs:', t1 != t3)
    print('CHECK test_comparison rhs:', True)
    assert (t1 != t3) == True


def test_chained_compare_with_empty_list_literal() -> None:
    xs: list[int] = [1, 2]
    ys: list[int] = [1, 2]
    out: bool = xs == [] == ys
    print('CHECK test_comparison lhs:', out)
    print('CHECK test_comparison rhs:', False)
    assert out == False


def run_tests() -> None:
    test_eq_true()
    test_eq_false()
    test_neq_true()
    test_neq_false()
    test_lt_true()
    test_lt_false()
    test_lt_equal()
    test_gt_true()
    test_gt_false()
    test_gt_equal()
    test_lte_less()
    test_lte_equal()
    test_lte_greater()
    test_gte_greater()
    test_gte_equal()
    test_gte_less()
    test_cmp_zero()
    test_cmp_negative()
    test_cmp_negative_ordering()
    test_cmp_with_arithmetic()
    test_cmp_variables()
    test_list_lexicographic_ordering()
    test_class_comparison_ordering_and_eq()
    test_tuple_eq_matrix()
    test_chained_compare_with_empty_list_literal()
