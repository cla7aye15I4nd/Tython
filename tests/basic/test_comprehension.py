class Countdown:
    current: int
    stop: int

    def __init__(self, start: int, stop: int) -> None:
        self.current = start
        self.stop = stop

    def __iter__(self) -> "Countdown":
        return self

    def __next__(self) -> int:
        if self.current < self.stop:
            raise StopIteration()
        value: int = self.current
        self.current = self.current - 1
        return value


def risky_divide(n: int, events: list[int]) -> int:
    result: int = 0
    try:
        events.append(10)
        if n == 0:
            raise Exception("division by zero")
        if n < 0:
            raise Exception("negative input")
        result = 12 // n
    except Exception:
        events.append(20)
        result = -1
    finally:
        events.append(30)
    return result


def test_try_except_finally_raise_nested() -> None:
    events: list[int] = []

    ok: int = risky_divide(3, events)
    bad_zero: int = risky_divide(0, events)
    bad_negative: int = risky_divide(-2, events)

    assert ok == 4
    assert bad_zero == -1
    assert bad_negative == -1

    assert events == [10, 30, 10, 20, 30, 10, 20, 30]

    nested_score: int = 0
    try:
        try:
            raise Exception("inner")
        except Exception:
            nested_score = nested_score + 5
            raise Exception("outer")
    except Exception:
        nested_score = nested_score + 11
    finally:
        nested_score = nested_score + 19

    assert nested_score == 35


def test_iter_next_stopiteration_manual_and_for() -> None:
    manual: Countdown = Countdown(4, 1)
    seen: list[int] = []

    while True:
        try:
            seen.append(manual.__next__())
        except StopIteration:
            break

    assert seen == [4, 3, 2, 1]

    loop_iter: Countdown = Countdown(5, 2)
    loop_sum: int = 0
    for value in loop_iter:
        loop_sum = loop_sum + value
    assert loop_sum == 14


def test_for_over_iterable_object_nested() -> None:
    outer: Countdown = Countdown(4, 1)
    total: int = 0
    products: list[int] = []

    for i in outer:
        for j in Countdown(i, 1):
            total = total + i * j
            products.append(i * j)

    assert products == [16, 12, 8, 4, 9, 6, 3, 4, 2, 1]
    assert total == 65


def test_list_and_tuple_comprehensions_nested() -> None:
    matrix: list[list[int]] = [[i * j for j in range(1, 5)] for i in range(1, 5)]
    assert matrix == [[1, 2, 3, 4], [2, 4, 6, 8], [3, 6, 9, 12], [4, 8, 12, 16]]

    flat_filtered: list[int] = [x for row in matrix for x in row if x % 3 == 0]
    assert flat_filtered == [3, 6, 3, 6, 9, 12, 12]

    tuple_items: list[tuple[int, int, tuple[int, int]]] = [
        (i, j, (i + j, i * j))
        for i in range(1, 4)
        for j in range(2, 5)
        if (i + j) % 2 == 0
    ]
    assert tuple_items == [
        (1, 3, (4, 3)),
        (2, 2, (4, 4)),
        (2, 4, (6, 8)),
        (3, 3, (6, 9)),
    ]

    tuple_from_comp: tuple[tuple[int, int], tuple[int, int], tuple[int, int]] = tuple(
        (n, n * n) for n in range(1, 4)
    )
    assert tuple_from_comp == ((1, 1), (2, 4), (3, 9))


def test_stopiteration_repeated_after_exhaustion() -> None:
    c: Countdown = Countdown(2, 1)
    seen: list[int] = []
    stop_hits: int = 0

    while True:
        try:
            seen.append(c.__next__())
        except StopIteration:
            stop_hits = stop_hits + 1
            break

    try:
        c.__next__()
    except StopIteration:
        stop_hits = stop_hits + 1

    assert seen == [2, 1]
    assert stop_hits == 2


def test_try_except_finally_with_nested_for_and_raise() -> None:
    total: int = 0
    except_hits: int = 0
    finally_hits: int = 0

    for i in range(1, 4):
        for j in Countdown(3, 2):
            try:
                if i == 2 and j == 3:
                    raise Exception("planned")
                total = total + i * j
            except Exception:
                except_hits = except_hits + 1
                total = total + 1
            finally:
                finally_hits = finally_hits + 1

    assert total == 25
    assert except_hits == 1
    assert finally_hits == 6


def test_iterator_consumption_in_comprehensions() -> None:
    shared: Countdown = Countdown(4, 1)
    first: list[int] = [x for x in shared]
    second: list[int] = [x for x in shared]

    assert first == [4, 3, 2, 1]
    empty: list[int] = []
    assert second == empty

    pair_tuple: tuple[
        tuple[int, int],
        tuple[int, int],
        tuple[int, int],
        tuple[int, int],
        tuple[int, int],
        tuple[int, int],
    ] = tuple((x, y) for x in Countdown(3, 1) for y in range(1, x + 1))
    assert pair_tuple == ((3, 1), (3, 2), (3, 3), (2, 1), (2, 2), (1, 1))

    diagonal_products: list[int] = [p[0] * p[1] for p in pair_tuple if p[0] == p[1]]
    assert diagonal_products == [9, 4, 1]


def test_deep_nested_comprehensions_with_filters() -> None:
    triples: list[list[tuple[int, int, int]]] = [
        [(i, j, i * j) for j in range(1, 5) if (i * j) % 2 == 0]
        for i in range(1, 5)
        if i != 3
    ]
    assert triples == [
        [(1, 2, 2), (1, 4, 4)],
        [(2, 1, 2), (2, 2, 4), (2, 3, 6), (2, 4, 8)],
        [(4, 1, 4), (4, 2, 8), (4, 3, 12), (4, 4, 16)],
    ]

    flattened: list[int] = [t[2] for row in triples for t in row if t[2] % 4 == 0]
    assert flattened == [4, 4, 8, 4, 8, 12, 16]


def test_internal_iter_next_basic() -> None:
    it: Countdown = iter(Countdown(4, 2))
    values: list[int] = []

    values.append(next(it))
    values.append(next(it))
    values.append(next(it))

    stop_hits: int = 0
    try:
        next(it)
    except StopIteration:
        stop_hits = stop_hits + 1

    assert values == [4, 3, 2]
    assert stop_hits == 1


def test_internal_iter_next_nested_structure() -> None:
    outer: Countdown = iter(Countdown(3, 1))
    values: list[int] = []
    total: int = 0

    while True:
        try:
            i: int = next(outer)
            inner: Countdown = iter(Countdown(i, 1))
            while True:
                try:
                    j: int = next(inner)
                    values.append(i * 10 + j)
                    total = total + i * j
                except StopIteration:
                    break
        except StopIteration:
            break

    assert values == [33, 32, 31, 22, 21, 11]
    assert total == 25


def run_tests() -> None:
    test_try_except_finally_raise_nested()
    test_iter_next_stopiteration_manual_and_for()
    test_for_over_iterable_object_nested()
    test_list_and_tuple_comprehensions_nested()
    test_stopiteration_repeated_after_exhaustion()
    test_try_except_finally_with_nested_for_and_raise()
    test_iterator_consumption_in_comprehensions()
    test_deep_nested_comprehensions_with_filters()
    test_internal_iter_next_basic()
    test_internal_iter_next_nested_structure()
