def test_list_literal_len_get_set() -> None:
    xs: list[int] = [10, 20, 30]
    assert len(xs) == 3
    assert xs[0] == 10
    assert xs[-1] == 30

    xs[1] = 99
    print('CHECK test_list_literal_len_get_set lhs:', xs[1])
    print('CHECK test_list_literal_len_get_set rhs:', 99)
    assert xs[1] == 99


def test_list_append_clear_pop() -> None:
    xs: list[int] = [1, 2]
    xs.append(3)
    print('CHECK test_list_append_clear_pop lhs:', xs)
    print('CHECK test_list_append_clear_pop rhs:', [1, 2, 3])
    assert xs == [1, 2, 3]

    last: int = xs.pop()
    print('CHECK test_list_append_clear_pop lhs:', last)
    print('CHECK test_list_append_clear_pop rhs:', 3)
    assert last == 3
    assert xs == [1, 2]

    xs.clear()
    assert len(xs) == 0


def test_list_insert_remove_index_count() -> None:
    xs: list[int] = [1, 2, 2, 4]
    xs.insert(1, 9)
    assert xs == [1, 9, 2, 2, 4]

    xs.remove(2)
    assert xs == [1, 9, 2, 4]

    idx: int = xs.index(9)
    cnt: int = xs.count(2)
    assert idx == 1
    assert cnt == 1


def test_list_extend_copy_reverse() -> None:
    xs: list[int] = [1, 2]
    ys: list[int] = [3, 4]
    xs.extend(ys)
    assert xs == [1, 2, 3, 4]
    assert ys == [3, 4]

    zs: list[int] = xs.copy()
    zs[0] = 100
    assert xs[0] == 1
    assert zs[0] == 100

    xs.reverse()
    assert xs == [4, 3, 2, 1]


def test_list_sort_for_supported_types() -> None:
    nums: list[int] = [5, 1, 4, 3, 2]
    nums.sort()
    assert nums == [1, 2, 3, 4, 5]

    words: list[str] = ["z", "b", "a"]
    words.sort()
    assert words == ["a", "b", "z"]


def test_list_contains_and_truthiness() -> None:
    xs: list[int] = [7, 8, 9]
    assert 8 in xs
    assert 100 not in xs

    assert xs
    ys: list[int] = []
    assert not ys


def test_list_methods_with_nested_function_and_comprehension() -> None:
    base: list[int] = [4, 1, 3, 2, 2]

    def transform(seed: list[int]) -> list[int]:
        xs: list[int] = seed.copy()
        xs.append(9)
        xs.insert(2, 7)
        xs.remove(1)
        assert xs.count(2) == 2
        assert xs.index(7) == 1
        popped: int = xs.pop()
        assert popped == 9
        xs.extend([8, 6])
        xs.reverse()
        xs.sort()
        assert xs == [2, 2, 3, 4, 6, 7, 8]

        evens: list[int] = [x * 10 for x in xs if x % 2 == 0]
        assert evens == [20, 20, 40, 60, 80]

        xs.clear()
        assert xs == []
        return evens

    out: list[int] = transform(base)
    assert out == [20, 20, 40, 60, 80]
    assert base == [4, 1, 3, 2, 2]


def test_list_crazy_nested_comprehensions() -> None:
    xs: list[int] = [1, 2, 3, 4]
    ys: list[int] = [2, 3, 5]

    matrix: list[list[int]] = [[x * y + x for y in ys] for x in xs]
    print('CHECK test_list matrix lhs:', matrix)
    print('CHECK test_list matrix rhs:', [[3, 4, 6], [6, 8, 12], [9, 12, 18], [12, 16, 24]])
    assert matrix == [[3, 4, 6], [6, 8, 12], [9, 12, 18], [12, 16, 24]]

    flat_filtered: list[int] = [
        value
        for row in matrix
        for value in row
        if value % 3 == 0
        if value % 4 != 0
    ]
    print('CHECK test_list flat_filtered lhs:', flat_filtered)
    print('CHECK test_list flat_filtered rhs:', [3, 6, 6, 9, 18])
    assert flat_filtered == [3, 6, 6, 9, 18]

    combos: list[int] = [
        a * 100 + b * 10 + c
        for a in xs
        for b in ys
        for c in [a + b, a * b]
        if (a + b + c) % 2 == 0
    ]
    print('CHECK test_list combos lhs:', combos)
    print('CHECK test_list combos rhs:', [123, 134, 156, 224, 224, 235, 257, 325, 336, 358, 426, 428, 437, 459])
    assert combos == [123, 134, 156, 224, 224, 235, 257, 325, 336, 358, 426, 428, 437, 459]


def run_tests() -> None:
    test_list_literal_len_get_set()
    test_list_append_clear_pop()
    test_list_insert_remove_index_count()
    test_list_extend_copy_reverse()
    test_list_sort_for_supported_types()
    test_list_contains_and_truthiness()
    test_list_methods_with_nested_function_and_comprehension()
    test_list_crazy_nested_comprehensions()

if __name__ == "__main__":
    run_tests()
