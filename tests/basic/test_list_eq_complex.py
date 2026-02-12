def test_list_eq_nested() -> None:
    a: list[list[int]] = [[1, 2], [3, 4]]
    b: list[list[int]] = [[1, 2], [3, 4]]
    c: list[list[int]] = [[1, 2], [3, 5]]
    assert a == b
    assert a != c
    print("list_eq_nested ok")

def test_list_eq_deep() -> None:
    a: list[list[list[int]]] = [[[1, 2]], [[3, 4]]]
    b: list[list[list[int]]] = [[[1, 2]], [[3, 4]]]
    c: list[list[list[int]]] = [[[1, 2]], [[3, 5]]]
    assert a == b
    assert a != c
    print("list_eq_deep ok")

def test_list_eq_str() -> None:
    a: list[str] = ["hello", "world"]
    b: list[str] = ["hello", "world"]
    c: list[str] = ["hello", "xyz"]
    assert a == b
    assert a != c
    print("list_eq_str ok")

def test_list_eq_tuple_inner() -> None:
    a: list[tuple[int, int]] = [(1, 2), (3, 4)]
    b: list[tuple[int, int]] = [(1, 2), (3, 4)]
    c: list[tuple[int, int]] = [(1, 2), (3, 5)]
    assert a == b
    assert a != c
    print("list_eq_tuple ok")

def test_list_neq_diff_len() -> None:
    a: list[int] = [1, 2, 3]
    b: list[int] = [1, 2]
    assert a != b
    print("list_neq_len ok")

def test_tuple_eq_nested() -> None:
    a: tuple[tuple[int, int], int] = ((1, 2), 3)
    b: tuple[tuple[int, int], int] = ((1, 2), 3)
    c: tuple[tuple[int, int], int] = ((1, 9), 3)
    assert a == b
    assert a != c
    print("tuple_eq_nested ok")

def run_tests() -> None:
    test_list_eq_nested()
    test_list_eq_deep()
    test_list_eq_str()
    test_list_eq_tuple_inner()
    test_list_neq_diff_len()
    test_tuple_eq_nested()
