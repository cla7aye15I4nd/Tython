class EqBox:
    value: int

    def __init__(self, value: int) -> None:
        self.value = value

    def __eq__(self, other: "EqBox") -> bool:
        return self.value == other.value


class LtBox:
    value: int

    def __init__(self, value: int) -> None:
        self.value = value

    def __lt__(self, other: "LtBox") -> bool:
        return self.value < other.value


def test_nested_list_eq_like_int() -> None:
    xs: list[list[int]] = [[1, 2], [3, 4], [1, 2]]
    target: list[int] = [1, 2]
    print('CHECK test_list_recursive_mono lhs:', target in xs)
    print('CHECK test_list_recursive_mono rhs:', True)
    assert (target in xs) == True
    print('CHECK test_list_recursive_mono lhs:', xs.index(target))
    print('CHECK test_list_recursive_mono rhs:', 0)
    assert xs.index(target) == 0
    print('CHECK test_list_recursive_mono lhs:', xs.count(target))
    print('CHECK test_list_recursive_mono rhs:', 2)
    assert xs.count(target) == 2
    xs.remove([3, 4])
    print('CHECK test_list_recursive_mono lhs:', xs)
    print('CHECK test_list_recursive_mono rhs:', [[1, 2], [1, 2]])
    assert xs == [[1, 2], [1, 2]]


def test_nested_list_eq_like_class() -> None:
    rows: list[list[EqBox]] = [[EqBox(1), EqBox(2)], [EqBox(3)]]
    a: list[EqBox] = [EqBox(1), EqBox(2)]
    b: list[EqBox] = [EqBox(3)]
    print('CHECK test_list_recursive_mono lhs:', a in rows)
    print('CHECK test_list_recursive_mono rhs:', True)
    assert (a in rows) == True
    print('CHECK test_list_recursive_mono lhs:', rows.index(a))
    print('CHECK test_list_recursive_mono rhs:', 0)
    assert rows.index(a) == 0
    print('CHECK test_list_recursive_mono lhs:', rows.count(a))
    print('CHECK test_list_recursive_mono rhs:', 1)
    assert rows.count(a) == 1
    rows.remove(b)
    print('CHECK test_list_recursive_mono lhs:', len(rows))
    print('CHECK test_list_recursive_mono rhs:', 1)
    assert len(rows) == 1
    print('CHECK test_list_recursive_mono lhs:', rows[0] == [EqBox(1), EqBox(2)])
    print('CHECK test_list_recursive_mono rhs:', True)
    assert (rows[0] == [EqBox(1), EqBox(2)]) == True


def test_nested_list_sort_and_sorted_class_lt() -> None:
    xs: list[list[LtBox]] = [
        [LtBox(3), LtBox(9)],
        [LtBox(1), LtBox(5)],
        [LtBox(2), LtBox(1)],
    ]
    xs.sort()
    print('CHECK test_list_recursive_mono lhs:', xs[0][0].value)
    print('CHECK test_list_recursive_mono rhs:', 1)
    assert xs[0][0].value == 1
    print('CHECK test_list_recursive_mono lhs:', xs[1][0].value)
    print('CHECK test_list_recursive_mono rhs:', 2)
    assert xs[1][0].value == 2
    print('CHECK test_list_recursive_mono lhs:', xs[2][0].value)
    print('CHECK test_list_recursive_mono rhs:', 3)
    assert xs[2][0].value == 3

    src: list[list[LtBox]] = [[LtBox(4), LtBox(0)], [LtBox(3), LtBox(9)]]
    out: list[list[LtBox]] = sorted(src)
    print('CHECK test_list_recursive_mono lhs:', src[0][0].value)
    print('CHECK test_list_recursive_mono rhs:', 4)
    assert src[0][0].value == 4
    print('CHECK test_list_recursive_mono lhs:', out[0][0].value)
    print('CHECK test_list_recursive_mono rhs:', 3)
    assert out[0][0].value == 3


def run_tests() -> None:
    test_nested_list_eq_like_int()
    test_nested_list_eq_like_class()
    test_nested_list_sort_and_sorted_class_lt()
