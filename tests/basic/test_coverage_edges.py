def helper_no_return_annotation(x: int):
    y: int = x + 1
    print(y)


def helper_with_no_import() -> int:
    return 7


class Edge:
    v: int

    def __init__(self, v: int) -> None:
        self.v = v


def make_edge(v: int) -> "Edge":
    return Edge(v)


def test_no_return_annotation_and_nested_import() -> None:
    def inner(v: int) -> int:
        return v + 1

    out: int = inner(5)
    print('CHECK test_coverage_edges lhs:', out)
    print('CHECK test_coverage_edges rhs:', 6)
    assert out == 6
    helper_no_return_annotation(1)
    print('CHECK test_coverage_edges lhs:', helper_with_no_import())
    print('CHECK test_coverage_edges rhs:', 7)
    assert helper_with_no_import() == 7
    e: Edge = make_edge(3)
    print('CHECK test_coverage_edges lhs:', e.v)
    print('CHECK test_coverage_edges rhs:', 3)
    assert e.v == 3


def test_raise_name_and_except_name() -> None:
    hit: int = 0
    try:
        raise ValueError
    except ValueError:
        hit = 1
    print('CHECK test_coverage_edges lhs:', hit)
    print('CHECK test_coverage_edges rhs:', 1)
    assert hit == 1


def test_assert_tuple_truthiness() -> None:
    print('CHECK test_coverage_edges assert expr:', '(1, 2)')
    assert (1, 2)


def test_chained_compare_and_print_list() -> None:
    x: bool = 1 < 2 < 3
    print('CHECK test_coverage_edges assert expr:', 'x')
    assert x

    ys: list[int] = [1, 2, 3]
    print(ys)


def test_tuple_bytes_compare_and_comprehension_filters() -> None:
    a: tuple[int, int] = (1, 1)
    b: tuple[int, int] = (1, 1)
    c: tuple[int, int] = (2, 1)
    print('CHECK test_coverage_edges lhs:', a)
    print('CHECK test_coverage_edges rhs:', b)
    assert a == b
    print('CHECK test_coverage_edges assert expr:', 'a != c')
    assert a != c

    one_arg: list[int] = [i for i in range(5) if i > 1 if i < 4]
    print('CHECK test_coverage_edges lhs:', one_arg)
    print('CHECK test_coverage_edges rhs:', [2, 3])
    assert one_arg == [2, 3]

    three_arg: list[int] = [i for i in range(1, 10, 3)]
    print('CHECK test_coverage_edges lhs:', three_arg)
    print('CHECK test_coverage_edges rhs:', [1, 4, 7])
    assert three_arg == [1, 4, 7]


def run_tests() -> None:
    test_no_return_annotation_and_nested_import()
    test_raise_name_and_except_name()
    test_assert_tuple_truthiness()
    test_chained_compare_and_print_list()
    test_tuple_bytes_compare_and_comprehension_filters()
